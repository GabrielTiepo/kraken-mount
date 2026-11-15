# Plano de evolução — Kraken Mount / FPKG direto em SSD SATA externo

**Data:** 2026-11-15  
**Estado:** planejamento; sem implementação ou homologação da versão modificada  
**Fork:** https://github.com/GabrielTiepo/kraken-mount  
**Upstream:** https://github.com/quer3q/kraken-mount  
**Base de referência:** Kraken `801b98e228f4b66df4de6a1d043307c7f498d86c`; AMPR `12e2f8e86be26bfb064979eb372b9d59d304992e`; ShadowMountPlus `3530d8d88ea1fa0d907545eda7ac95d4240a0213`  
**Licença:** GPL-3.0-or-later, preservar atribuições e avisos.

## 1. Objetivo e cenário funcional

Evoluir a experiência do Kraken Mount usando o ShadowMountPlus como referência de biblioteca, backports, perfis e diagnósticos, **preservando o diferencial de abrir diretamente FPKGs debug sem instalar/descompactar todo o jogo**.

**Regressão obrigatória:** NBA 2K27, Title ID `PPSA35889`, FPKG base de cerca de **107 GB**, tamanho lógico informado pela ferramenta ~398 GB, backport em pasta ~158 MB, executado a partir de **SSD SATA externo conectado por USB**. O usuário confirmou que **o jogo iniciou**; ainda não confirmou partida completa, salvamento, carregamento repetido ou estabilidade prolongada. Não publicar arquivos do jogo, PKGs, saves ou backports protegidos no GitHub.

**Fora do escopo imediato:** cópia integral do ShadowMountPlus, instalação de SMB, alteração de firmware, formatação/alteração de cluster do SSD e promessa de aumentar FPS. Priorizar latência de I/O, tempos de carregamento, robustez do cache e compatibilidade.

## 2. Estado atual verificado no código

- `crates/kraken-mount-pkg`: lê FIH/CNT em FPKGs debug, descreve arquivos e offsets de blocos Kraken; **rejeita explicitamente LIH**, retail/encrypted e layouts não suportados.
- `src/ampr_index.rs`: gera índices `ampr_emu.index`/`ampr_assets.index`; blocos Kraken normalmente de até **256 KiB**, com página de leitura de **64 KiB**; a medição citada no código não representa automaticamente SSD SATA via USB.
- `src/cache.rs`: extrai `eboot.bin`, `sce_sys/**` e módulos PRX/SPRX; indexa os dados grandes e aplica backport em pasta. Arquivos esparsos incompatíveis com a grade podem exigir extração integral.
- Mudanças do AMPR podem atualizar só `fakelib`, mas mudanças no backport podem provocar rebuild abrangente de índice.
- `src/scan.rs`: descobre FPKG e pastas, associa backports por Title ID e atualmente prefere pasta ao PKG duplicado. Ainda não oferece filtro persistente `pkg/folder/all`.
- `src/ui.html` e `src/i18n.rs` **já têm tradução pt-BR**; revisar cobertura e experiência, não reimplementar do zero.
- `src/main.rs` pausa varredura/rebuild/purge enquanto um jogo está executando e espera por KStuff e pela ausência de ShadowMountPlus.
- `ampr-emu-ng` realiza cache de blocos decodificados, I/O assíncrona e possui instrumentação de debug. Seus testes de leitura publicados são primordialmente em arquivos soltos no SSD interno. **Readahead prejudicou latência de cauda em cenários medidos e permanece desativado por padrão.**
- Runtime PKG lê o arquivo base como volume 0, resolvendo offsets pelo índice; a arquitetura atual não suporta simplesmente anexar um segundo PKG como se fosse o mesmo volume.

## 3. Regras de arquitetura, segurança e rollout

1. **Zero regressão:** conservar PKG + backport em pasta; novas capacidades opt-in/desligadas por padrão, salvo mudança interna comprovadamente segura.
2. **Fonte imutável:** nunca reescrever o PKG ou o backport de origem. Atualizações em cache com staging, validação e swap atômico/rollback.
3. **Separação de responsabilidades:** leitor FPKG; cache/índices; fontes/overlays; runtime AMPR; montagem/registro; interface/API. Evitar acoplar otimização de runtime à UI.
4. **Montagens seguras:** sem alteração, rebuild, purge ou desmontagem forçada com jogo em execução; respeitar KStuff, conflito com ShadowMountPlus e BackPork.
5. **Armazenamento removível:** distinguir USB desconectado de exclusão do jogo; não limpar saves/caches automaticamente; recuperar quando o volume retornar.
6. **Validação defensiva:** não ignorar checagens de offset, bounds, formato, checksum, caminho e uso de memória para aceitar um arquivo incompatível.
7. **Retrocompatibilidade:** `config.toml` atual é TOML simplificado; defaults novos devem preservar a biblioteca e o comportamento existente.
8. **Segurança da API:** limitar exposição de dados e operações sensíveis à LAN confiável; novas rotas de escrita requerem validação e prevenção de acesso indevido; exclusões irreversíveis exigem confirmação.
9. **Performance baseada em evidências:** baseline A/B com no mínimo 3 execuções equivalentes; comparar mediana, p95/p99, CPU/RAM, erros e bytes físicos lidos, não só throughput sequencial.
10. **Release:** `main` representa base estável, `develop` integração; desenvolver via PRs em lotes e não substituir automaticamente o ELF comprovadamente funcional.

## L00 — Fundação, CI e baseline funcional

**Tarefas**
- Verificar fork/vínculo upstream; preservar `main` e criar `develop`; manter referência do SHA original e estratégia de sincronização com upstream.
- Revisar `.github/workflows/build.yml` e `ps5/build.sh`; reproduzir build Rust/FreeBSD Docker, checksum SHA-256 e release beta explícita.
- Criar fixtures sintéticas **pequenas** para FIH/CNT, raw, Kraken, esparsos, arquivos truncados, caminhos inválidos e duplicatas; documentar testes possíveis no host.
- Documentar cenário do NBA 2K27: versão de ELF, KStuff, runtime, backport, USB, cache e logs sem incluir conteúdos de jogos/saves.
- Medir abertura do PKG, criação de índice, tempo até menu, primeira partida, salvamento e relançamento após reboot; registrar o que ainda estiver indisponível.

**Aceite:** CI verde; build ELF reproduzível; jogo original + backport ainda inicia no setup real, **ou marcado como NÃO HOMOLOGADO se teste no PS5 não foi executado**.

**Áreas:** `.github/workflows`, `ps5/`, `AGENTS.md`, `DESIGN.md`, testes do parser.

## L01 — Interface pt-BR, controle de formatos e biblioteca

**Tarefas**
- Revisar termos brasileiros, mensagens de erro, idioma selecionável/persistente e notificações.
- `source_mode = "all" | "pkg" | "folder"` com default `all`; filtrar na descoberta e **manter backports em pasta associados aos PKGs**.
- Preferência explícita para duplicatas de mesmo Title ID: não priorizar pasta automaticamente quando um FPKG funcional foi escolhido.
- Varredura `automatic` ou `manual`; no manual manter monitoramento de saúde de fontes ativas e disponibilizar rescan explícito.
- Cadastro de fonte por caminho permitido, sem copiar/excluir o arquivo; validar symlink, raízes, permissões, Title ID e estado da origem.
- Biblioteca com filtros, versão, tipo, drive, tamanho PKG/cache, backport, erros explicativos e ações reversíveis; desativar formato não apaga jogo/cache/save/tile silenciosamente.

**Aceite:** pt-BR consistente; PKG+backport funciona em `source_mode=pkg`; alterações não apagam arquivos nem removem jogos sem solicitação.

**Áreas:** `src/config.rs`, `src/scan.rs`, `src/main.rs`, `src/web.rs`, `src/ui.html`, `src/i18n.rs`.

## L02 — Backports, fakelib e perfis por jogo

**Tarefas**
- Preferências persistentes por Title ID: caminho e ativação de backport, seleção de fonte, fakelib padrão/completa/somente emuladores/desativada e opção AMPR por jogo.
- Interface de visualizar, instalar, substituir e remover backport **em pasta** com comparação de Title ID, cópia atômica e sem apagar a origem.
- Definir precedência determinística entre jogo, backport, `fakelib2`, `fakelib` e AMPR; evitar sobrescrita involuntária de biblioteca específica.
- Aviso e diagnóstico quando ShadowMountPlus/BackPork puderem gerar conflito.

**Aceite:** padrão atual mantém NBA 2K27 funcional; perfil de um título não modifica outros; rollback sem apagar dados.

**Áreas:** `src/cache.rs`, `src/launch.rs`, `src/register.rs`, `src/config.rs`, `src/web.rs`, `src/ui.html`.

## L03 — Cache incremental e recuperação de falhas

**Tarefas**
- Separar índice base imutável, overlay de backport e composição de fakelib/AMPR.
- Mudança apenas de backport **não** recalcula o índice completo do PKG; refazer somente overlay e referências necessárias.
- Índices atualizados em diretório temporário/staging, integridade validada antes de ativação; preservar cache anterior caso falhe.
- Resolver mtime de Windows/exFAT que causa rebuild contínuo; usar fingerprint confiável para identidade rápida, distinguindo de integridade criptográfica completa.
- Relatar espaço usado por cache e ações distintas `rebuild`, `refresh overlay`, `refresh fakelib` e `purge` (destrutivo).

**Aceite:** trocar backport não reindexa dados grandes; USB removido ou falha de energia não elimina versão anterior válida.

**Áreas:** `src/cache.rs`, `src/ampr_index.rs`, `src/main.rs`, `src/web.rs`, `DESIGN.md`.

## L04 — Diagnóstico e otimização de SSD SATA externo via USB

**Instrumentação:** medir tempo de abertura do PKG, leitura do descritor, build de cache, tempo até menu/partida, leituras físicas/lógicas, latência p50/p95/p99, acertos de cache, descompressão, fila AIO, EIO/timeout e reconexão. Exibir painel *Diagnóstico de leitura* em pt-BR com dados reais e `indisponível` quando não houver informação.

**Experimentos — SEM alterar defaults**
- Medir configuração existente de páginas **64 KiB**, comparando 16/128/256 KiB **somente quando o runtime permitir**, sem confundir páginas de I/O com clusters do exFAT.
- Testar read-ahead `off` versus janelas pequenas (1–2 chunks), somente com telemetria de uso real; não habilitar globalmente sem evidência.
- Ajustar concorrência, janela e fila de I/O para p95/p99 e ausência de travamentos, não para throughput máximo.
- Medir SSD, adaptador USB-SATA, cabo/porta e filesystem; não presumir velocidade de USB 3.0, UASP, TRIM ou cache do hardware.
- Exportação de relatório sem caminhos pessoais completos, segredos ou dados de jogos.

**Gate de performance:** 3 ou mais amostras por cenário e A/B no mesmo jogo/dispositivo, com resultados documentados. Rejeitar mudança que cause piora consistente em p95/p99, erros ou instabilidade. **Nunca formatar o disco como parte da otimização.**

**Áreas:** `src/web.rs`, `src/ui.html`, `src/main.rs`, `src/launch.rs` e, se necessário, uma tarefa independente em fork versionado do `ampr-emu-ng`.

## L05 — Leitor FPKG e arquivos grandes/esparsos

**Tarefas**
- Corrigir casos de sparse holes desalinhados sem extrair arquivo enorme por padrão: investigar materialização seletiva de blocos problemáticos em volume auxiliar.
- Erros distintos para FIH, LIH, CNT, retail/encrypted, layout/shuffle, truncamento, mídia ausente e patch incompatível.
- Fixtures de boundary, fuzz, overflow, offsets, nomes inválidos e limites de memória; não remover validação.
- Variantes de layout somente com fixture comprovada e limites explícitos de memória/CPU.

**Aceite:** falha segura em pacote inválido; fallback documentado; nenhum arquivo grande extraído inesperadamente; jogos atuais sem regressão.

**Áreas:** `crates/kraken-mount-pkg/src/`, `src/cache.rs`, `src/ampr_index.rs`.

## L06 — PKG base + update/backport em segundo PKG, FIH/LIH

**Atenção:** retirar a rejeição a LIH **não é suficiente**. O patch pode fazer referência a blocos do pacote base. Resolver estrutura e dependências antes de montar.

**Fases**
1. Identificação de base/update/backport por Title ID, Content ID, tipo, versão, dependência; recusar incompatibilidade antecipadamente.
2. Para dois FPKGs independentes suportados, resolver overlay `base -> update -> backport` com fontes imutáveis. Investigar multi-pack no AMPR e reindexação incremental.
3. Estudar layout LIH com fixtures sintéticas, leitura dos blocos do pacote base e falhas explícitas em casos desconhecidos.
4. Interface para associar/remover update, alternar versão e rollback; proibir extração indiscriminada de ~398 GB.

**Aceite:** update incompatível falha com segurança; base segue funcionando sozinha; dois PKGs compatíveis executam sem alterar arquivos de origem nem exigir extração completa; regressão L00 preservada.

**Áreas:** `kraken-mount-pkg`, `src/scan.rs`, `src/cache.rs`, `src/ampr_index.rs`, `src/launch.rs`, `src/web.rs`; runtime AMPR separado se necessário.

## L07 — Homologação, distribuição beta e contribuição upstream

**Matriz:** PKG com e sem backport; pasta plain/LZ4/ZSTD; SSD interno e SATA USB; reconexão/ausência USB; reboot e retomada; arquivos truncados; duplicate Title IDs; cache insuficiente; mudança de perfis; atualização.

- Respeitar jogos em execução, sem `MNT_FORCE` em register/purge; garantir segurança de montagens e diretórios.
- CI + testes de parser/cache + build Docker/ELF, artefato com SHA-256, changelog, notas beta e procedimento de rollback.
- Release beta somente após revisão; não substituir ELF original automaticamente.
- Relatório pré/pós de performance com números medidos ou declaração explícita de inconclusivo.
- Rever contribuições pequenas ao upstream, preservando créditos e GPL.

**Aceite:** zero regressões; gate de hardware concluído ou status **NÃO HOMOLOGADO**; nenhuma promessa de ganho sem teste real.

## 4. Dependências e ordem recomendada

`L00` primeiro. Depois, **instrumentação do L04 -> L03 -> L02 -> L01 -> experimentos medidos L04 -> L05 -> L06 -> L07**. L01 e L02 podem evoluir de forma independente após L00. LIH não deve começar antes de estabilizar o caminho funcional de leitura e cache.

## 5. Checklist obrigatório de PR

- [ ] PR de escopo contido para `develop`, com riscos, reversão e migração documentados.
- [ ] Checks do host, testes sintéticos e build PS5 via Docker; não declarar hardware testado só porque CI passou.
- [ ] `DESIGN.md` atualizado quando a arquitetura/contrato mudar; pt-BR revisado.
- [ ] Sem escrita em PKG/backport original; sem excluir saves, títulos ou caches sem autorização.
- [ ] Testes de regressão de leitura FPKG, pasta, LZ4, ZSTD e backport.
- [ ] Se desempenho: A/B medido, p50/p95/p99, erros, bytes e ambiente USB documentados.
- [ ] NBA 2K27: confirmar registro, menu, partida, salvamento, reinicialização e relançamento no PS5, quando houver acesso ao hardware.
- [ ] Se não ocorreu teste real, declarar **NÃO HOMOLOGADO NO PS5**.

## 6. Fontes

- Código Kraken: https://github.com/quer3q/kraken-mount
- Design: https://github.com/quer3q/kraken-mount/blob/main/DESIGN.md
- Runtime: https://github.com/quer3q/ampr-emu-ng
- Runtime PKG: https://github.com/quer3q/ampr-emu-ng/blob/main/crates/ampr-sys/src/rt/pack/pkg.rs
- Estudo de I/O: https://github.com/quer3q/ampr-emu-ng/blob/main/docs/read-path-tuning.md
- ShadowMountPlus: https://github.com/drakmor/ShadowMountPlus

**Status:** plano publicado no fork; implementação, benchmark e homologação ainda não realizados.
