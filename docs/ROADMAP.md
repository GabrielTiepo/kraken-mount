# Roadmap — Kraken Mount (fork GabrielTiepo)

**Status:** planejamento registrado; **nenhum lote implementado ou homologado**.  
**Plano canônico:** [PLANO-KRAKEN-MOUNT-SATA-SSD.md](./PLANO-KRAKEN-MOUNT-SATA-SSD.md)  
**Base funcional conhecida:** NBA 2K27 `PPSA35889` inicializou a partir de FPKG (~107 GB) + backport em pasta, no SSD SATA externo/USB. Partida, save e estabilidade prolongada ainda não comprovados.

| Lote | Tema | Estado | Dependência / gate |
|---|---|---|---|
| L00 | Baseline, CI, testes e build ELF | ☐ Pendente | Primeiro; manter referência ao ELF atual |
| L01 | pt-BR, biblioteca, formatos PKG/pasta | ☐ Pendente | L00 |
| L02 | Backport, fakelib e perfis por jogo | ☐ Pendente | L00 |
| L03 | Cache incremental, atualização atômica e falhas USB | ☐ Pendente | L00; ideal antes de L06 |
| L04 | Diagnóstico e A/B de desempenho no SSD SATA USB | ☐ Pendente | Baseline L00; recomendações somente com medição |
| L05 | Compatibilidade FPKG e arquivos esparsos | ☐ Pendente | Fixtures L00 e estabilidade do cache |
| L06 | FPKG base + segundo PKG (FIH/LIH) | ☐ Pendente | L03/L05; investigar runtime AMPR |
| L07 | Homologação geral, beta ELF e rollout | ☐ Pendente | L00–L06 implementados conforme escopo |

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

A tentativa de criar a Issue épica foi recusada pelo GitHub: **HTTP 410 — "Issues has been disabled in this repository."**

Para habilitar: no fork, abra **Settings → General → Features → Issues** e marque **Issues**. Quando habilitado, criar:
- uma Issue épica: "Evolução do Kraken Mount — FPKG direto, experiência ShadowMountPlus e SSD SATA USB";
- oito Issues L00–L07 com escopo, dependências e aceite da seção correspondente do plano;
- rastreio de PRs para `develop` e links de CI/homologação.

**Este roadmap é o acompanhamento provisório até as Issues estarem habilitadas.**

## Referências

- Fork: https://github.com/GabrielTiepo/kraken-mount
- Upstream: https://github.com/quer3q/kraken-mount
- AMPR: https://github.com/quer3q/ampr-emu-ng
- Referência funcional: https://github.com/drakmor/ShadowMountPlus
