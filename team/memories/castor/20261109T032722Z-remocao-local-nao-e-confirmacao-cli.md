---
type: AgentMemory
title: "Remoção bilateral local ainda não é confirmação da CLI"
description: "A PR #531 tem RED/GREEN e E2E de releitura após POST sintético, mas --remover --confirmar continua sem cobertura fim a fim."
agent: castor
timestamp: "2026-11-09T03:27:22Z"
tags: [rust, relacionamentos, bdd, anti-ies, sqlite, causalidade]
---

# Ponto de retomada

**Fato:** a [PR #531](https://github.com/franklinbaldo/ies/pull/531) concluiu uma fatia parcial do BDD-258-03: POST de exclusão **local** no anti-IES remove somente a aresta bilateral, preserva vizinhos e a CLI real lê depois o SQLite alterado via TLS loopback. [RED #42553799133](https://github.com/franklinbaldo/ies/actions/runs/42553799133): 2 falhas e 6 acertos antes da implementação; [GREEN #42555531155](https://github.com/franklinbaldo/ies/actions/runs/42555531155): 7/7 no head `af18db29`, com `main@ad5b46c7` e `behind=0` naquela consulta. PR Ready, ainda aberta. O [handoff está na #258](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6731512000).

**Limite:** [Actions #42554375244](https://github.com/franklinbaldo/ies/actions/runs/42554375244) conferiu integridade da cassette PEN1 (25 interações), não equivalência semântica. Treinamento NÃO EXECUTADO. `--remover --confirmar` segue bloqueado antes da rede, sem escrita externa. Próxima sessão: conferir integração, então completar BDD-258-02/03/04 com confirmação da CLI, POST único e releitura independente.

**Cafezinho:** a [conversa sobre contrafólio](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6731328214) destacou que observação adicional só é útil quando diferencia hipóteses causais; analogia da ficção, não teste de papel nem do SEI.
