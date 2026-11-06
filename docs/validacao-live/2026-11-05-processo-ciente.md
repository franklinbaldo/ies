---
type: SEIObservation
title: Estado de validação de ciência de processo e documento em 05/11/2026
description: Evidências offline e pendências live da issue 271
---

# Ciência — validação e resíduos (05/11/2026)

| Ensaio | Status | Requisições |
| --- | --- | --- |
| Testes do leitor sintético, RED | FAIL como esperado | 0 SEI |
| Testes sintéticos e integração na main (fatia anterior) | PASS no CI da PR #280; `processo ciente` tem 3 GET de prévia e 7 GET no caso feliz sintético | 0 SEI |
| RED da regressão causa do GET mutador — [42215714854](https://github.com/franklinbaldo/ies/actions/runs/42215714854) | FAIL esperado: **3 falhas novas, 1.711 PASS, 22 ignorados preexistentes**; as falhas são HTTP 403, Infra 200 e transporte (testes antes do código) | 0 SEI |
| GREEN da preservação de causa | CI do novo head a verificar; não declarar PASS sem conclusão | 0 SEI |
| Replay íntegro das cassettes do Todos | NOT_RUN | 0 SEI |
| Prévia UnB (treinamento) | NOT_RUN | 0 |
| Prévia em segunda instalação (treinamento) | NOT_RUN | 0 |
| `--confirmar` em treinamento | NOT_RUN | 0 |
| Releitura de confirmação ao vivo | NOT_RUN | 0 |
| Escritas e resíduos novos | 0 / 0 | 0 |

Nenhuma escrita foi executada e **nenhum resíduo foi deixado no SEI**.
A execução ao vivo exige acesso comprovado a instalação de
treinamento, prévia no próprio processo, um único `--confirmar`
e verificação por leitura. Não usar produção. O CI só prova o
comportamento sintético, não a compatibilidade ao vivo.
