---
type: AgentMemory
title: "Confirmação CLI só no debug .test, com aceite e releitura"
description: "A PR #538 executa adição, remoção e efeito incerto pela CLI real contra anti-IES/SQLite, preservando a trava do release."
agent: castor
timestamp: "2026-11-09T05:17:30Z"
tags: [bdd, relacionamentos, rust, causalidade, anti-ies, sqlite]
---

# Próxima junta

**Fato:** a [PR #538](https://github.com/franklinbaldo/ies/pull/538) retoma #258 sem repetir prévias: os BDD-258-02/03/04 agora passam **sinteticamente** pela CLI real contra anti-IES/SQLite em TLS loopback, na origem reservada `https://sei.test/sei` e apenas com binário debug. POST modificador único + GET nova; se o backend escreve mas a resposta é 200 inconclusiva, saída 3 sem retry. **Release e ambientes externos continuam bloqueados antes da rede.**

[RED #42565052365](https://github.com/franklinbaldo/ies/actions/runs/42565052365): 3 falhas / 4 acertos no E2E antes do código. Após ajustes de Clippy e Rustfmt, [CI oficial final #42566148781](https://github.com/franklinbaldo/ies/actions/runs/42566148781) **7/7 PASS**, head merge verdadeiro `0e6f2d8c`, `behind=0` antes desta memória. A [inspeção PEN1 #42565036347](https://github.com/franklinbaldo/ies/actions/runs/42565036347) validou hashes de 25 interações, **não** paridade semântica; treinamento **NOT_RUN**, zero requisições/resíduos.

**Retomar:** revisão 🦅, integração 🐘/🦡 da #538; validação em treinamento autorizado exige prévia, confirmação única e releitura independente antes de remover a trava pública. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a discussão do contrafólio continua lembrando que o estado final não distingue sozinho duas histórias causais; isso é analogia, não evidência do SEI.
