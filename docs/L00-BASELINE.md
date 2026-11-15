# L00 — Baseline funcional e protocolo de regressão

Estado: **baseline preliminar**; resultado do console relatado pelo usuário, **nenhum teste novo foi executado pelo projeto neste fork**.
Plano: [PLANO-KRAKEN-MOUNT-SATA-SSD.md](./PLANO-KRAKEN-MOUNT-SATA-SSD.md) · [Issue L00 #2](https://github.com/GabrielTiepo/kraken-mount/issues/2)

## 1. Origem versionada e ponto de retorno

| Componente | Referência |
|---|---|
| Kraken Mount upstream | `quer3q/kraken-mount`, commit `801b98e228f4b66df4de6a1d043307c7f498d86c` |
| Branch estável do fork | `main`, **não atualizada automaticamente** |
| Integração | `develop` |
| Execução de desenvolvimento | Branches de curta duração, PRs para `develop` |
| Runtime AMPR | `quer3q/ampr-emu-ng` (código-fonte separado; a biblioteca distribuída pelo Kraken é empacotada em `vendor/ampr-emu-ng`) |
| Backports | Pasta separada por Title ID; nunca sobrescrever o PKG de origem |

**Rollback:** manter cópia de `kraken-mount.elf` que já iniciou o jogo e do `libSceAmpr.sprx` efetivamente usado no console, com SHA-256 e localização anotada *fora* do repositório. A cópia original deve permanecer utilizável até concluir L07. Não substituir automaticamente o payload em funcionamento.

## 2. Cenário informado — NBA 2K27

| Campo | Informação disponível |
|---|---|
| Title ID | `PPSA35889` |
| Formato | FPKG debug (`.pkg`) carregado diretamente pelo Kraken Mount |
| Tamanho físico informado | ~107 GB |
| Tamanho lógico informado por ferramenta | ~398 GB (não extraído integralmente) |
| Backport | Pasta separada, ~158 MB |
| Armazenamento | SSD SATA externo ligado ao PS5 via USB |
| Resultado confirmado pelo usuário | **O jogo iniciou** |
| Não confirmado | Partida completa, tempos, salvamento, reboot, relançamento, travamentos, p95/p99, transferência USB |

Não publicar PKG, executáveis de jogos, backports, saves, credenciais, IP ou caminhos pessoais em Issues/CI. Registrar somente metadados não sensíveis.

## 3. Baseline técnico do leitor

O reader `crates/kraken-mount-pkg` valida FIH/CNT, rejeita LIH e outros layouts não suportados e fornece blocos Kraken para a geração do índice AMPR. O cache em `/data/krakenmount/caches/<TID>` mantém `eboot.bin`, módulos e metadados necessários. O `ampr-emu-ng` atende leituras do PKG em tempo de execução, sem extrair todos os dados.

**Unidades distintas:** uma página de I/O de 64 **KiB** (`src/ampr_index.rs`) não é o tamanho do *cluster do filesystem* do SSD. Blocos de dados Kraken podem ter 256 KiB. Não recomendar formatação do SSD como “otimização”.

## 4. Verificações automatizadas disponíveis neste lote

```bash
# Verificação do payload no alvo FreeBSD/PS5 (compilação cruzada)
cargo check --workspace --locked --target x86_64-unknown-freebsd

# Testes reais no host Linux do parser puro, com FIH/CNT sintéticos (sem arquivos de jogo)
cargo test -p kraken-mount-pkg --lib --locked

# Verificação de sintaxe JS da UI
sed -n '/^<script>$/,/^<\/script>$/p' src/ui.html | sed '1d;$d' > /tmp/kraken-ui.js
node --check /tmp/kraken-ui.js

# Construção completa em Docker: produz target/ps5/kraken-mount.elf
./ps5/build.sh
sha256sum target/ps5/kraken-mount.elf
```

O CI executa check, testes do parser e validação JS antes de gastar tempo construindo o ELF. O build PS5 usa duas etapas (a segunda inclui o ELF de autoinicialização). O workflow disponibiliza o payload com `SHA256SUMS`; criação de release permanece restrita a tags `v*`, inicialmente como draft.

**Cobertura do novo conjunto de testes:** FIH válido, FIH truncado ou malformado, PS4/LIH/retail rejeitados, CNT válido, entradas fora do pacote, tabela excessiva, erros de I/O e leituras fragmentadas. **Limitação:** essas fixtures não emulam PFS completo, decodificação Kraken, AMPR, execução no console ou um FPKG comercial real.

## 5. Teste físico a executar antes da promoção

Manter o mesmo jogo/backport, porta USB, adaptador, armazenamento e runtime. Não testar em paralelo com ShadowMountPlus, e não realizar purge ou mudanças de montagem durante a execução.

| Etapa | Versão funcional | Versão do PR | Registro |
|---|---|---|---|
| Identificação do PKG e Title ID | Confirmar | Confirmar | Passou / Falhou / Não testado |
| Construção do cache sem extração integral | Confirmar | Confirmar | Tempo e bytes reais |
| Chegar ao menu inicial | Relatado: iniciou | Confirmar | Tempo |
| Abrir e completar uma partida | Não confirmado | Confirmar | Tempo / falhas |
| Salvar e carregar progresso | Não confirmado | Confirmar | Resultados |
| Sair e reabrir | Não confirmado | Confirmar | Tempo |
| Reiniciar console e reabrir | Não confirmado | Confirmar | Resultado |
| Desconectar USB *com jogo fechado*, reconectar e rescanner | Não confirmado | Confirmar | Sem perda dos arquivos |
| Falha por PKG removido/truncado (fixture separada) | Não testado | Validar | Erro seguro |

Para otimizações do L04, coletar **no mínimo 3 repetições A/B** com p50/p95/p99 de leituras, erros, bytes físicos, tempo até menu e até partida, antes de declarar melhora.

## 6. Critério de conclusão do L00

- [ ] CI verde e pacote ELF verificável pelo SHA-256;
- [ ] fixtures do parser aprovadas no host Linux;
- [ ] documentação e versão estável de rollback preservadas;
- [ ] validação física do NBA 2K27 (ou evidência explícita **NÃO HOMOLOGADO NO PS5**);
- [ ] nenhum pacote de jogo ou save enviado ao GitHub.

**Importante:** aprovação automatizada não autoriza substituir o ELF em uso ou promover alterações de comportamento para `main`.
