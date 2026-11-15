# L03 — Incremento 2: refrescar fakelib do backport sem reindexar o FPKG

**Escopo:** `Fresh::Fakelib` para alteração **somente** de biblioteca de backport em FPKGs.
**Status:** alteração em branch, aguardando CI e teste no PS5. Não representa conclusão do L03.
**Referências:** [L03 #5](https://github.com/GabrielTiepo/kraken-mount/issues/5), [roadmap](./ROADMAP.md).

## Problema observado no código

`cache::inputs` inclui a pasta inteira do backport no caso de PKG. A alteração de um único
`fakelib/*.sprx` deixava o cache `Stale`, provocando `clear_build`, nova abertura/parsing
do PKG, extrações de arquivos, geração dos índices AMPR e registro, mesmo sem mudança
nos arquivos do jogo. O AMPR não indexa `fakelib/`: ela é a sobreposição de bibliotecas no launch.

## Solução limitada, com fallback seguro

1. Cada build novo de FPKG com backport acessível grava no stamp `.done` dois inventários:
   `backport_data` (arquivos de jogo fora de fakelib/fakelib2) e `backport_lib`
   (arquivos da biblioteca selecionada por precedência).
2. Inventários são ordenados por caminho relativo e contêm tamanho e mtime com nanos.
   Não armazenam dados do jogo. Não são hashes criptográficos nem capturam alterações
   com mesmo tamanho e timestamp preservado.
3. Ao verificar se o cache está atualizado, o caminho rápido só é liberado quando:
   mesmo PKG/source/tamanho/backport, base FPKG não foi modificada após o `.done`,
   `pkg_id` amostrado coincide, `eboot.bin`, `sce_sys/param.json`, índices AMPR e
   `.built` ainda existem, e o inventário de arquivos do jogo é **idêntico**.
4. Se **apenas** a biblioteca escolhida ou seus arquivos mudou, retorna
   `Fresh::Fakelib` e executa `refresh_fakelib` em vez de `build_inner`.
   Se o backport possui `fakelib2` ou `fakelib` de origem, o refresh não chama
   `Pkg::open`/não percorre blocos comprimidos. Continua copiando o AMPR conforme
   a configuração já existente.
5. Cache anterior sem snapshots, symlink detectado, caminho inválido, I/O de
   inventário com erro, 4.096+ arquivos, PKG modificado ou dado de jogo alterado:
   **não usa fast path; preserva o rebuild normal**.
6. A data `.done` do refresh é capturada **antes** da cópia para sinalizar
   entradas mudadas durante a recomposição, em vez de reaproveitar o mtime antigo.

## Cenários e resultados esperados

| Cenário | Esperado |
|---|---|
| Backport `fakelib/libA.sprx` substituída, `eboot.bin` inalterado | Somente `fakelib/` é recomposta; índices mantidos |
| `fakelib/` virou `fakelib2/` | Somente `fakelib/` é recomposta; sem indexação |
| Arquivo novo, excluído ou alterado fora da biblioteca | Rebuild completo (conservador) |
| PKG diferente, tamanho igual, mas mtime recente | Rebuild completo |
| Cache anterior sem `backport_data`/`backport_lib` | Rebuild completo |
| Alteração apenas do AMPR e backport com libs próprias | Recompõe biblioteca sem reabrir PKG |
| Arquivo de biblioteca é symlink | Fast path não habilitado |
| FPKG sem backport | Caminho original (AMPR/fakelib e rebuild) |
| Jogo em execução | Scanner não executa alterações de cache |

## Testes

```sh
rustc --edition=2021 --deny warnings --test src/backport_inventory.rs -o /tmp/kraken-backport-inventory
/tmp/kraken-backport-inventory
cargo check --workspace --locked --target x86_64-unknown-freebsd
./ps5/build.sh
```

O workflow `build.yml` integra os cinco testes de inventário ao CI:
separação fakelib/fakelib2, mudança somente de biblioteca, mudança de dados,
pasta vazia e rejeição de symlink. O `cargo check` é necessário também para
validar o uso da API no `src/cache.rs`.

**Limitações:** não implementa refresh de `eboot.bin` ou outros módulos sem reindex;
não realiza staging transacional/rollback da `fakelib`; não elimina toda
necessidade de reler o PKG. Durante o fast path, `pkg_id` ainda lê aproximadamente
128 KiB amostrados do arquivo; não percorre todo o FPKG. Não é benchmark de I/O
do AMPR na partida.

## Gate para homologação em SSD SATA USB

- [ ] CI FreeBSD, teste de inventário e ELF verde, artifact com SHA-256
- [ ] PS5 com mesmo PKG NBA 2K27 (`PPSA35889`) e backport — não tocar em arquivos de origem
- [ ] Registrar tempos e hashes dos índices AMPR antes e após *somente* mudança de uma biblioteca
- [ ] Confirmar que índices não foram reconstruídos; `fakelib` atualizou; jogo abriu e salvou
- [ ] Alterar **outro arquivo** do backport e confirmar que caiu no rebuild conservador
- [ ] Repetir com USB desconectado e após reboot, sem purge; preservar ELF anterior

**Não homologado no PS5 até execução física comprovada.**
