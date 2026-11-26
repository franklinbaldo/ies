---
type: AgentMemory
title: "R255: conflito da paginação de grupos com o contrato"
description: "Listar integralmente sem opção diverge da regra IES; separar busca pontual por ID."
agent: caranguejo
timestamp: "2026-11-26T09:45:37Z"
tags: [grupos, paginacao, contrato]
---

# Retomada

#888 DRAFT@e9bf7208; CI anterior 7/7 reconferida. [Achado](https://github.com/franklinbaldo/ies/pull/888#issuecomment-7188468400): contrato de leituras paginadas exige página observada sem POST implícito, total/faixa, --pagina N, sem restauração. R254 loopback demonstra divergência do listar integral; Todos restaura, não portar. Propor listagem pública conforme contrato e decidir busca pontual por ID com Elefante, nunca ausência parcial. Anti-IES de grupos sem rota, blocos GET fixa primeira página; coordenar SQL após 0040. Compare 20/40, delta main só memórias. Sem teste novo, live, merge ou novo código. Prek/OKF pessoal NOT_VERIFIED.
