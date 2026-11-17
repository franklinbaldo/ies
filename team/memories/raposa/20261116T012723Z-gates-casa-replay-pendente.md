---
type: AgentMemory
title: "🦊 #851: gate da casa não substitui replay Rust"
description: "Snapshot privado da casa validado em árvore exata, com dois negativos; #852 ainda aguarda replay nominal e CI oficial."
agent: raposa
timestamp: "2026-11-16T01:27:23Z"
tags: [ies, raposa, documento-salvar, prek, ci, cassette]
---

# Retomar

**Fato novo:** recuperei o SSH do Ifro e reconstruí a casa `franklinbaldo/raposa@f216c8e` a partir do tree `971c87db73db97f8cd1f4f0b9d7ec04e5c07d466`. Pre-push instalado, `prek==0.5.10` **2/2 PASS**; em cópias isoladas, tamanho 20.001 bytes e ausência de frontmatter falharam só em seus respectivos hooks (`rc=1`). A memória da casa foi publicada no commit [71e7103](https://github.com/franklinbaldo/raposa/commit/71e710381d7834155fb561ad72ab8ba36bbde126). Isso não é prova da CLI.

**IES:** a [PR #852](https://github.com/franklinbaldo/ies/pull/852) segue Draft com ablação de detectores redundantes. A revisão [Gavião](https://github.com/franklinbaldo/ies/pull/852#pullrequestreview-5913848982) requer teste Rust nominal no HEAD, negativos e CI/replay oficial. A execução de Rust foi retomada por executor autorizado mas seu resultado ainda era **NOT_VERIFIED** nesta observação; IES↔cassette e anti-IES↔cassette independentes, CLI↔anti-IES↔SQLite específico do salvar e live seguem **NOT_RUN**. Zero GET/POST externo.

**Próximo passo:** preservar branch única, conferir os logs completos do teste nominal e replays oficiais no SHA final; não promover inspeção de ZIP a paridade nem liberar escrita remota. Cafezinho [mesa #20](https://github.com/franklinbaldo/cafezinho/issues/20), [piloto #5](https://github.com/franklinbaldo/cafezinho/issues/5) e IES [#388](https://github.com/franklinbaldo/ies/issues/388) consultados.
