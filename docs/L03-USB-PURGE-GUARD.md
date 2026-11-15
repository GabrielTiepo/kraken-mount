# L03 — Bloqueio preventivo de purge com mídia USB desconectada

**Status:** código implementado no fork; **não compilado e não homologado no PS5**.  
[Issue L03 #5](https://github.com/GabrielTiepo/kraken-mount/issues/5) · [Plano de evolução](PLANO-KRAKEN-MOUNT-SATA-SSD.md).

## Problema

A limpeza de cache anterior recebe uma lista de títulos disponíveis na última varredura.
Um SSD SATA USB desconectado desaparece da biblioteca e seus títulos deixam de entrar no
conjunto de preservação; uma limpeza solicitada pelo usuário poderia então tratar esses
caches como órfãos. Isso não deve acontecer por uma desconexão temporária.

## Comportamento implementado

O `cache::purge` agora chama `purge_media_preflight` **antes** de qualquer remoção:

1. Consulta o conjunto de aliases montados em `/mnt/usb0..7` e `/mnt/ext0..1` pelo próprio `storage.rs`.
2. Percorre os diretórios de caches e lê os campos `source` das estampas `.done` válidas.
3. Para fontes sob montagens removíveis conhecidas, confirma a presença exata da unidade.
4. Se faltar uma, aborta o purge inteiro com `WouldBlock` e informa a unidade a reconectar.
5. Se uma estampa existente estiver ilegível ou malformada, bloqueia por segurança. Pastas sem `.done` são ignoradas pela checagem (dados parcialmente construídos, sem fonte rastreável).

**A operação só age sobre os caches; não escreve no PKG, backport ou save.** Também não introduz custo de CPU ou I/O no caminho de leitura do AMPR: a checagem ocorre **somente** quando o usuário pede Purge.

## Limitações explícitas

- Verifica a unidade montada, não a identidade física do SSD; um dispositivo diferente
  montado no mesmo alias não é detectado.
- Uma desconexão *durante* o purge pode ocorrer após o preflight. A proteção não resolve
  essa corrida; operações desmontagem, locks e consistência devem evoluir nos próximos incrementos.
- Um cache sem `.done` não contém a identidade da fonte e não pode ser protegido por este mecanismo.
- Essa proteção é específica de **Purge**. A exclusão intencional de um jogo e o rebuild têm
  contratos diferentes e devem receber revisões próprias.
- Não implementa ainda o cache incremental de backports nem snapshot/rollback atômico de índices.

## Testes e aceite

```sh
rustc --edition=2021 --deny warnings --test src/removable_guard.rs -o /tmp/removable-tests
/tmp/removable-tests
```

Os seis testes puros cobrem mídia ausente/presente, USB/ext, dados em SSD interno,
nomes ambíguos de montagens e ausência total de volumes. O workflow
`.github/workflows/removable-cache-guard.yml` chama esse comando nos PRs.

- [ ] CI do fork executado e aprovado (sem execuções verificadas na última consulta)
- [ ] Build ELF do FreeBSD/PS5 aprovado, respeitando todos os gates do L00
- [ ] PS5: testar com SSD SATA conectado (Purge disponível)
- [ ] PS5: desconectar somente com jogo fechado; verificar que o cache não foi eliminado pelo Purge
- [ ] PS5: reconectar a mesma unidade; verificar que o título retorna sem extração integral
- [ ] PS5: validar NBA 2K27, FPKG e backport, com o ELF antigo preservado para rollback

**Aprovação:** NÃO HOMOLOGADO NO PS5. Não substituir automaticamente o payload funcional.
