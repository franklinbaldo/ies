---
type: AgentMemory
title: "Releitura de remoção precisa preservar identidade e vizinhos"
description: "RED demonstrou falso sucesso por id desaparecido e perda colateral de relações; GREEN tipado exige NUP ausente e demais vínculos preservados."
agent: castor
timestamp: "2026-11-09T00:14:28Z"
tags: [rust, bdd, relacionamentos, causalidade, anti-ies, red-green]
---

# Próximo encaixe

**Fato:** após integração da [#491](https://github.com/franklinbaldo/ies/pull/491), retomei [#258](https://github.com/franklinbaldo/ies/issues/258) na [PR #520](https://github.com/franklinbaldo/ies/pull/520). O [RED #42538183896](https://github.com/franklinbaldo/ies/actions/runs/42538183896) teve **2 falhas e 2.121 testes aprovados**: o motor dizia `Confirmado` se o id antigo desaparecia, apesar do mesmo NUP reaparecer sob outro id ou outra relação ter sumido. Agora a releitura tipada exige destino ausente e **demais relações preservadas**, sem outro POST. Três regressões incluem o caminho positivo. [GREEN #42538724143](https://github.com/franklinbaldo/ies/actions/runs/42538724143): **7/7 PASS** no head `6b812bff`, reconciliado por merge verdadeiro da main (behind=0). PR Ready, não mesclada por Castor.

**Três provas:** motor Rust sintético PASS; inspeção de integridade da cassette PEN1 em [Actions #42538396742](https://github.com/franklinbaldo/ies/actions/runs/42538396742) PASS (25 interações), mas **confronto semântico NOT_RUN**; treinamento **NOT_RUN**. A escrita `--confirmar` da CLI permanece bloqueada; não declarar BDD-258-03 E2E. Próximo ciclo: consultar integração da PR antes de criar trabalho, depois BDD-258-02/03/04 já previstos.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6729174458), lembrei da junta de madeira: a ausência do pino antigo não comprova a soltura do encaixe, e perder uma travessa vizinha não é remoção correta. Metáfora de identificabilidade, **não observação do SEI**.
