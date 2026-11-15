# Roadmap — Kraken Mount (fork GabrielTiepo)

**Status:** upstream v0.2.0 integrado via PR #13. Código L00 e incrementos L01/L03/L04 integrados à `develop` com CI aprovado; L01/L03/L04 ainda possuem escopo pendente. Nenhum lote homologado no PS5.  
**Plano canônico:** [PLANO-KRAKEN-MOUNT-SATA-SSD.md](./PLANO-KRAKEN-MOUNT-SATA-SSD.md)  
**Base funcional conhecida:** NBA 2K27 `PPSA35889` inicializou a partir de FPKG (~107 GB) + backport em pasta, no SSD SATA externo/USB. Partida, save e estabilidade prolongada ainda não comprovados.

| Lote | Tema | Estado | Dependência / gate |
|---|---|---|---|
| [L00](https://github.com/GabrielTiepo/kraken-mount/issues/2) | Baseline, CI, testes e build ELF | **Código integrado (PR #10); compilação do payload validada no CI após sync, PS5 pendente** | Primeiro; manter referência ao ELF atual |
| [L01](https://github.com/GabrielTiepo/kraken-mount/issues/3) | pt-BR, biblioteca, formatos PKG/pasta | **Parcial — [PR #11](https://github.com/GabrielTiepo/kraken-mount/pull/11) integrado e CI verde**; funcionalidades adicionais/PS5 pendentes | L00 |
| [L02](https://github.com/GabrielTiepo/kraken-mount/issues/4) | Backport, fakelib e perfis por jogo | ☐ Pendente | L00 |
| [L03](https://github.com/GabrielTiepo/kraken-mount/issues/5) | Cache incremental, atualização atômica e falhas USB | **Parcial — [PR #14](https://github.com/GabrielTiepo/kraken-mount/pull/14) integrado e CI verde**; cache incremental/PS5 pendentes | L00; ideal antes de L06 |
| [L04](https://github.com/GabrielTiepo/kraken-mount/issues/6) | Diagnóstico e A/B de desempenho no SSD SATA USB | **Parcial — [PR #12](https://github.com/GabrielTiepo/kraken-mount/pull/12) integrado e CI verde**; métricas AMPR em jogo/PS5 pendentes | Baseline L00; recomendações somente com medição |
| [L05](https://github.com/GabrielTiepo/kraken-mount/issues/7) | Compatibilidade FPKG e arquivos esparsos | ☐ Pendente | Fixtures L00 e estabilidade do cache |
| [L06](https://github.com/GabrielTiepo/kraken-mount/issues/8) | FPKG base + segundo PKG (FIH/LIH) | ☐ Pendente | L03/L05; investigar runtime AMPR |
| [L07](https://github.com/GabrielTiepo/kraken-mount/issues/9) | Homologação geral, beta ELF e rollout | ☐ Pendente | L00–L06 implementados conforme escopo |

## Integração oficial v0.2.0

- [PR #13 — sincronização oficial integrada](https://github.com/GabrielTiepo/kraken-mount/pull/13) em `develop`.
- [Documento de integração](./UPSTREAM-V0.2.0-INTEGRATION.md): histórico da release, AMPR v0.4.1, atualização do AMPR e registro `/system_ex` somente leitura.
- [PR #11](https://github.com/GabrielTiepo/kraken-mount/pull/11) (L01) foi adaptado à v0.2.0, validado por CI e **integrado**, mantendo `ampr_auto_update`.
- [PR #12](https://github.com/GabrielTiepo/kraken-mount/pull/12) (L04) foi adaptado à v0.2.0, aprovado pelo CI e **integrado à `develop`** após o PR #11.
- [PR #14](https://github.com/GabrielTiepo/kraken-mount/pull/14) (L03) foi validado por CI e **integrado**; protege caches quando uma unidade USB/ext previamente indexada não está montada.
- GitHub Actions passou nos PRs #11, #12 e #14 (checks e Docker ELF). **Nenhum ELF personalizado foi homologado no PS5**; a `main` continua preservada.

## Ordem prática recomendada

1. L00: CI/fixtures/build baseline e procedimento de rollback.
2. L04 (instrumentação): medir p50/p95/p99, bytes físicos, tempos de tela/partida e erros no SSD SATA USB.
3. L03: separar índices imutáveis do PKG de overlays de backport/fakelib.
4. L02 e L01: perfis de compatibilidade, interface, pt-BR e filtro PKG/pasta.
5. L04 (experimentos): página I/O 64 KiB vs alternativas, readahead off/on e paralelismo **somente se métricas justificarem**.
6. L05 e L06: ampliar leitor e suporte a segundo PKG, preservando regras de validação de LIH.
7. L07: teste físico, distribuição beta e revisão upstream.

## Critérios gerais de aceite

- [ ] Nenhuma alteração ou exclusão de PKGs, backports de origem ou saves.
- [ ] Nenhuma operação de rebuild/purge/desmontagem forçada durante execução do jogo.
- [ ] Cache resistente a falhas e desconexão/reconexão USB.
- [ ] Build e testes automatizados aprovados, com logs e hashes de artefato.
- [ ] NBA 2K27 permanece inicializando; validar partida, salvamento e relançamento no PS5 antes de marcar homologado.
- [ ] Toda alegação de melhoria de I/O possui baseline e medições A/B.
- [ ] Se hardware real não foi testado, registrar **NÃO HOMOLOGADO NO PS5**.
- [ ] Documentação `DESIGN.md`, português e licença GPL preservados.

## Acompanhamento via GitHub Issues

**Issues habilitadas e criadas em 2026-11-15.**

- [Épico principal #1](https://github.com/GabrielTiepo/kraken-mount/issues/1)
- [Issue #2 — L00](https://github.com/GabrielTiepo/kraken-mount/issues/2)
- [Issue #3 — L01](https://github.com/GabrielTiepo/kraken-mount/issues/3)
- [Issue #4 — L02](https://github.com/GabrielTiepo/kraken-mount/issues/4)
- [Issue #5 — L03](https://github.com/GabrielTiepo/kraken-mount/issues/5)
- [Issue #6 — L04](https://github.com/GabrielTiepo/kraken-mount/issues/6)
- [Issue #7 — L05](https://github.com/GabrielTiepo/kraken-mount/issues/7)
- [Issue #8 — L06](https://github.com/GabrielTiepo/kraken-mount/issues/8)
- [Issue #9 — L07](https://github.com/GabrielTiepo/kraken-mount/issues/9)

Acompanhar PRs direcionados a `develop`, check runs e evidências de homologação nas respectivas issues. **Issue aberta não significa implementação concluída.**


## Referências

- Fork: https://github.com/GabrielTiepo/kraken-mount
- Upstream: https://github.com/quer3q/kraken-mount
- AMPR: https://github.com/quer3q/ampr-emu-ng
- Referência funcional: https://github.com/drakmor/ShadowMountPlus
