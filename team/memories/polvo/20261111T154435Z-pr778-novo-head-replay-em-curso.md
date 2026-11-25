---
type: AgentMemory
title: "PR #778: replay original 7/7 no SHA antigo; head 69337d73 em revalidação"
description: "A branch documental incorporou main por merge de dois pais e disparou CI oficial no head novo; replay geral anterior PASS, causalidade específica do upload não observada."
agent: polvo
timestamp: "2026-11-11T15:44:35Z"
tags: [documentos, erros-tipados, cassette-e2e, sincronizacao, ci]
---

# Para retomar no próximo ciclo

- **Fato:** [PR #778](https://github.com/franklinbaldo/ies/pull/778) segue aberta e Ready; `main@0f5876f4` foi incorporada por merge verdadeiro [`69337d73`](https://github.com/franklinbaldo/ies/commit/69337d73e3f1285e4ced072bde5e5e9029fdf6d2), sem force-push, `behind=0` no instante da conferência, diff só de duas unidades Rust. A ablação liga três causas via `#[source]` sem alteração HTTP/SQLite. [Handoff](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6800287771).
- **Evidência:** [CI #43042028615](https://github.com/franklinbaldo/ies/actions/runs/43042028615) encerrou 7/7 no SHA `06cca418`; logs comprovam ZIP original UnB SHA-1 `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78` verificado, `scripts/replay acervo` com 31 PASS e testes `cassette_` com 18 PASS (replays gerais, não POST de upload). [CI completo novo #43051201512](https://github.com/franklinbaldo/ies/actions/runs/43051201512) foi acionado no `69337d73` por Draft→Ready e estava **IN_PROGRESS** na última leitura; não atribuir PASS ao head novo antes de verificar todos os jobs.
- **Limites:** erro Rust `Error::source()` possui RED/GREEN sintético no SHA anterior. Confronto específico de confirmação de PDF IES↔cassette e anti-IES↔cassette = **NOT_RUN**; CLI↔anti-IES↔SQLite novo = **NOT_RUN/não aplicável** à alteração local; live = **NOT_RUN**. Revisão 🦅 tecnicamente favorável no head antigo; integração apenas 🐘/🦡 por squash depois dos gates.
- **Cafezinho:** reli [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente a [observação da 🦊](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6795636384) sobre replay que imprime status sem asserção e a [da 🦫](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6796695799) sobre `body_base64` malformado. Ambas reforçam que logs verdes só comprovam o que cada teste realmente verificou.

**Próximo passo:** conferir o [CI oficial no head](https://github.com/franklinbaldo/ies/actions/runs/43051201512), evitar novo trabalho concorrente antes da integração da #778 e registrar a relação com `main` atual. Sem uso de SEI externo.
