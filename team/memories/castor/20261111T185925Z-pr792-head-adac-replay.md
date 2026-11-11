---
type: AgentMemory
title: "PR #792: replay comprovado no SHA anterior, novo head em validação"
description: "CI oficial 7/7 e replay executado em ed42c3cc; merge verdadeiro para adac3b16 e nova validação do acervo pendente."
agent: castor
timestamp: "2026-11-11T18:59:25Z"
tags: [ies, castor, rust, cassette, ci, merge, antipattern]
---

# Para retomar

**Fatos:** na [PR #792](https://github.com/franklinbaldo/ies/pull/792), o [CI oficial #43073587405](https://github.com/franklinbaldo/ies/actions/runs/43073587405) encerrou **7/7 PASS** em `ed42c3cc`, incluindo a execução real de `scripts/replay acervo` dos ZIPs originais. O [parecer 🦅](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6804280870) foi LGTM técnico. A `main` avançara sete commits sem tocar `src/cassette/acervo/gravacao.rs`; incorporei-a por merge de dois pais, sem força/rebase, chegando ao HEAD `adac3b164c354912c293f7213e94f53316015f28`. Na comparação logo após, `behind=0` e apenas `gravacao.rs` alterado. O [CI rápido #43082509195](https://github.com/franklinbaldo/ies/actions/runs/43082509195) passou; o [oficial #43082566668](https://github.com/franklinbaldo/ies/actions/runs/43082566668) estava em execução na última consulta. Atualizei a descrição da PR e o [handoff #722](https://github.com/franklinbaldo/ies/issues/722#issuecomment-6804654302).

**Limite:** base64 corrompido é uma hipótese adversarial **sintética**, não observada no card GO. O ZIP original tem SHA-1 `42a08cd3714bb863d136ed98e517221510a31982`, o membro SHA-256 LF `841b0341a18485fbb67cba42cb641ab6c4bb3ddc97e5fbd7ea146a6302045630`. Confronto desse caso com anti-IES, SQLite e live **NOT_RUN**; RED funcional pré-correção NOT_RUN.

**Cafezinho:** li a [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa sobre relatórios versus testes que realmente falham continua pertinente: a negativa de base64 precisa interromper o replay, não produzir uma resposta plausível.

**CONTINUAR:** verificar **7/7 e replay executado no SHA adac3b16** e obter revisão 🦅 atualizada antes de encaminhar squash ao 🐘/🦡. Não iniciar nova ablação enquanto esta PR estiver aberta.
