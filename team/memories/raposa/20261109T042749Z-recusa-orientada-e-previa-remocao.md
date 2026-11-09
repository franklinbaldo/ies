---
type: AgentMemory
title: "Atribuição: recusa orientada e prévia de remoção pela CLI real"
description: "PR #536 verde 7/7 separa prévia, escrita interna comprovada e bloqueio público, com E2E de remoção em anti-IES/SQLite."
agent: raposa
timestamp: "2026-11-09T04:27:49Z"
tags: [cli, ux, atribuicao, anti-ies, bdd, cassette, seguranca]
---

# Para retomar

**Fato:** [PR #536](https://github.com/franklinbaldo/ies/pull/536) (head `7f06d1f`) implementou recusa de `--confirmar` com alternativa explícita para prévia, em texto humano e erro JSON tipado legado. O texto da confirmação interna só afirma "Atribuição confirmada"/"Remoção confirmada" quando o estado tipado é `AlteracaoConfirmada`. Não liberar `--confirmar` público: a trava continua antes de sessão/rede. Testes de **CLI real ↔ HTTPS local ↔ anti-IES ↔ SQLite** cobrem a prévia de `--remover`, com 5 pedidos, nenhum POST de atribuição e estado persistido inalterado.

**Prova:** RED [CI #42560721054](https://github.com/franklinbaldo/ies/actions/runs/42560721054) = 1 falha de mensagem ausente; GREEN final [CI #42561674883](https://github.com/franklinbaldo/ies/actions/runs/42561674883) = 7/7 PASS e CI rápido [#42561667220](https://github.com/franklinbaldo/ies/actions/runs/42561667220) PASS. Handoff na [issue #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6732241130). Merge verdadeiro da `main` preservou memória do Castor e branch terminou 0 commits atrás **antes desta memória**.

**Limites:** integridade de cassettes UnB/GO comprovada em [Actions anterior #42555280812](https://github.com/franklinbaldo/ies/actions/runs/42555280812); confronto independente dos corpos HTML **NOT_RUN** nesta fatia; treinamento/live **NOT_RUN**, nenhuma escrita externa. BDD-264-02/03 completos ainda pendentes. Não confundir prova sintética de estado interno com confirmação externa.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6732067850), 🦫 relembrou que uma tabela ausente numa página observada difere de jamais observar a página. Na UI, a palavra "confirmada" também precisa de testemunha própria: não nasce de um estado desejado. Analogia de projeto, não evidência SEI.

**Próximo:** revisão 🦅 e integração 🐘/🦡 da PR, mantendo #264 aberta. Avançar confronto semântico independente e ensaio em TREINAMENTO com prévia, POST único, releitura, remoção e resíduo, sempre sem retry de efeito incerto.
