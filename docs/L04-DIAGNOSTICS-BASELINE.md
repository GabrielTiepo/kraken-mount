# L04 — Primeiro baseline de diagnóstico (SSD SATA USB)

**Status:** incremento de instrumentação implementado em branch; CI, build ELF e teste real no PS5 **não executados**.  
Issue: [#6](https://github.com/GabrielTiepo/kraken-mount/issues/6) · Plano: [L00–L07](./PLANO-KRAKEN-MOUNT-SATA-SSD.md)

## 1. O que é medido agora

O Kraken registra na memória do processo:

| Campo | Unidade | O que significa |
|---|---|---|
| `diagnostics.scan.elapsed_ms` | milissegundos | Tempo de uma execução de `scan::scan`, incluindo varredura de raízes, metadados e eventual abertura do PKG para leitura de título |
| `diagnostics.scan.games` | jogos | Fontes únicas descobertas |
| `diagnostics.scan.duplicates` | jogos | Fontes duplicadas descartadas na seleção |
| `diagnostics.scan.errors` | erros | Entradas que falharam na varredura |
| `diagnostics.builds[].title_id` | ID | Título cuja operação de cache terminou |
| `diagnostics.builds[].drive` | caminho de montagem | Origem em `/data` ou `/mnt/usb*`/`/mnt/ext*` |
| `diagnostics.builds[].kind` | `pkg` ou `folder` | Tipo de fonte |
| `diagnostics.builds[].elapsed_ms` | milissegundos | Tempo total de `cache::build` (leitura do PKG, criação de índices, extrações, escrita no SSD interno) |
| `diagnostics.builds[].success` | booleano | Resultado final da operação de cache |

O campo `diagnostics.scope` é sempre `"management"`. `scan` começa como `null` até a primeira varredura. `builds` contém **até oito resultados recentes**, em ordem cronológica, e começa vazio. As métricas são reiniciadas ao reiniciar o ELF, sem arquivos de log adicionais e sem tocar nos PKGs de origem.

**Limitação fundamental:** não medimos I/O do AMPR durante as partidas, descompressão por bloco, cache hit rate, read-ahead, throughput USB ou FPS. Construir um cache em 20 segundos não significa que o SSD externo consegue ler determinado valor em MB/s: a operação inclui CPU, alocação e escrita no SSD interno.

## 2. API e tela

`GET http://<ip-do-ps5>:8097/api/state` contém o novo objeto:

```json
{
  "diagnostics": {
    "scope": "management",
    "scan": null,
    "builds": []
  }
}
```

O JSON acima é **a estrutura inicial de exemplo**, não uma leitura real de console.

Na tela **Configurações → Diagnóstico de varredura e cache**, a seção recolhida exibe a última varredura e o último build apenas quando disponíveis; caso contrário, `Ainda não medido`. Em outros idiomas sem novas traduções o sistema utiliza inglês como fallback.

## 3. Testes automáticos

```sh
rustc --edition=2021 --deny warnings --test src/diagnostics.rs -o /tmp/kraken-diagnostics-test
/tmp/kraken-diagnostics-test
node scripts/test-diagnostics-ui.cjs
```

Os testes do módulo puro validam contadores, resultado de erro, conversão milissegundos e janela deslizante de oito builds. O smoke da UI examina a função verdadeira do HTML, incluindo estado vazio, resultado anterior versus resultado mais recente e distinção entre sucesso/falha.

A matriz FreeBSD/Docker continua no workflow `.github/workflows/build.yml` e deve ser aprovada antes da promoção.

## 4. Próxima fase: desempenho real durante a partida

Depende de investigação adicional no runtime `ampr-emu-ng`, **em repositório separado e sem substituir o runtime original**:

- Contabilizar leituras físicas efetivas do arquivo do PKG, bytes, erros e latências p50/p95/p99 por bloco.
- Expor as métricas por um canal seguro para o gerenciador, com impacto baixo, limites de memória, nenhum payload de conteúdo dos jogos e nenhuma escrita no PKG.
- Registrar tempo desde iniciar o jogo até menu e partida, com procedimento manual reproduzível e pelo menos três repetições.
- Testar `readahead`, blocos e concorrência **apenas com A/B**, mantendo default antigo se houver piora na cauda de latência, estabilidade ou integridade.

## 5. Critérios de homologação

- [ ] `cargo check --workspace --locked --target x86_64-unknown-freebsd` sem avisos
- [ ] Testes unitários e smoke UI aprovados no Actions
- [ ] `ps5/build.sh` gera ELF íntegro com SHA-256 verificável
- [ ] Instalar o novo ELF em ambiente de teste, preservando o payload original
- [ ] No PS5: validar NBA 2K27, `PPSA35889`, ~107 GB FPKG + backport em pasta ~158 MB; inicialização e uma partida inteira; sem alteração de PKG nem save
- [ ] Comparar baseline antes/depois de cache build e varredura sem prometer melhorias não comprovadas
- [ ] Validar caso do SSD SATA USB ausente (nunca confundir ausência com necessidade de apagar dados)

**Sem testes reais no PS5, esta entrega permanece NÃO HOMOLOGADA NO HARDWARE.**
