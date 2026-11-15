# Integração do upstream — Kraken Mount v0.2.0

- **Upstream release:** https://github.com/quer3q/kraken-mount/releases/tag/v0.2.0
- **Upstream commit:** `73cc894b88338b617dfe8b173585e94db78cd348`
- **Fork anterior:** `801b98e228f4b66df4de6a1d043307c7f498d86c`
- **Estratégia:** merge com dois pais, preservando o histórico oficial e as alterações exclusivas do fork
- **Destino:** `develop`. **Não atualizar `main` nem substituir ELF funcional automaticamente.**

## O que veio do upstream
- `ampr-emu-ng v0.4.1` no `vendor/` (binary + `SHA256SUMS` + `VERSION` oficiais).
- Verificação e atualização do AMPR via `src/net.rs`, `src/main.rs`, API/UI e `ampr_auto_update` (default upstream: `true`).
- Correção para remontar `/system_ex` em configurações em que está somente leitura (elfldr isolado).
- Interface mais leve, com remoção do plano de fundo e melhor carregamento de capas.
- Ajustes no registro, notificações, compatibilidade e documentação oficial.

## Preservado do nosso fork
- `.github/workflows/build.yml` com testes do leitor FIH/CNT no host e build Docker condicionado a checks.
- Testes unitários adicionais em `crates/kraken-mount-pkg/src/tests.rs`, `lib.rs` e `kraken.rs`.
- `docs/PLANO-KRAKEN-MOUNT-SATA-SSD.md`, `docs/ROADMAP.md` e `docs/L00-BASELINE.md`.
- `AGENTS.md` com entrada `src/net.rs` do upstream e instruções dos nossos testes.
- Nada do upstream foi copiado parcialmente: todos os outros arquivos da árvore são idênticos à `v0.2.0`, inclusive os binários versionados.

## Pontos de atenção

1. **AMPR auto-update ativado por padrão.** Na homologação do NBA 2K27, para manter a versão fixa e permitir A/B reproduzível, configurar temporariamente `ampr_auto_update = false`. A release inclui 0.4.1, mas a atualização automática pode obter uma versão posterior. Não remover biblioteca colocada manualmente nem atualizar o AMPR sem registrar a versão testada.
2. **LIH continua não suportado**, apesar do anúncio amplo de FPKG; veja `crates/kraken-mount-pkg/src/pkg.rs`. L05/L06 permanecem abertos.
3. **Sparse desalinhado pode extrair arquivo inteiro** para SSD interno (caminho atual em `src/cache.rs`); não foi solucionado nesta sincronização.
4. PRs [#11](https://github.com/GabrielTiepo/kraken-mount/pull/11) (L01) e [#12](https://github.com/GabrielTiepo/kraken-mount/pull/12) (L04) são anteriores à atualização oficial e precisam de adaptação e testes específicos em `src/config.rs`, `src/main.rs`, `src/web.rs` e `src/ui.html`.
5. **Evolução da validação:** o GitHub Actions foi habilitado após o primeiro merge e aprovou os PRs #11, #12 e #14, inclusive check FreeBSD, testes e build Docker/ELF. Veja o [run #43762225138](https://github.com/GabrielTiepo/kraken-mount/actions/runs/43762225138). Isso não equivale à homologação no PS5; conservar o ELF funcional para rollback.

## Gate antes de qualquer release personalizada

- [ ] `cargo check --workspace --locked --target x86_64-unknown-freebsd`
- [ ] `cargo test -p kraken-mount-pkg --lib --locked`
- [ ] `node --check` para o script da interface
- [ ] Build Docker `ps5/build.sh` com SHA-256 de `kraken-mount.elf`
- [ ] Testar NBA 2K27, `PPSA35889`, FPKG+backport, SSD SATA USB, pelo menos menu, partida, save e relançamento
- [ ] Provar que o cache anterior é preservado após atualização e que não há operação destrutiva sobre fonte de jogos

## Situação após a integração dos incrementos (15/11/2026)

- [PR #13](https://github.com/GabrielTiepo/kraken-mount/pull/13): upstream v0.2.0 integrado.
- [PR #14](https://github.com/GabrielTiepo/kraken-mount/pull/14): proteção dos caches de mídia removível integrada e com CI verde.
- [PR #11](https://github.com/GabrielTiepo/kraken-mount/pull/11): filtros PKG/pasta integrados e com CI verde.
- [PR #12](https://github.com/GabrielTiepo/kraken-mount/pull/12): diagnóstico de scan/cache integrado e com CI verde.
- `main` não foi alterada; **nenhum ELF do fork foi homologado em hardware**.
