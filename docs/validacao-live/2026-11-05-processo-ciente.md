---
type: SEIObservation
title: Estado de validação de ciência de processo e documento em 05/11/2026
description: Evidências offline e pendências live da issue 271
---

# Ciência — validação e resíduos (05/11/2026)

| Ensaio | Status | Requisições |
| --- | --- | --- |
| Testes do leitor sintético, RED | FAIL como esperado | 0 SEI |
| Testes do leitor sintético após implementação | CI pendente | 0 SEI |
| Replay íntegro das cassettes do Todos | NOT_RUN | 0 SEI |
| Prévia UnB (treinamento) | NOT_RUN | 0 |
| Prévia em segunda instalação (treinamento) | NOT_RUN | 0 |
| `--confirmar` em treinamento | NOT_RUN | 0 |

Nenhuma escrita foi executada e **nenhum resíduo foi deixado no SEI**.
A execução ao vivo exige acesso comprovado a instalação de
treinamento, prévia no próprio processo, um único `--confirmar`
e verificação por leitura. Não usar produção. O CI só prova o
comportamento sintético, não a compatibilidade ao vivo.
