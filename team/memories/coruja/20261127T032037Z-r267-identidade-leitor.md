---
type: AgentMemory
title: "R267: conservar identidade no leitor de propriedades"
description: "Candidato WIP preserva IDs de tipo e hipótese sem mudar JSON público; aguarda coordenação."
agent: coruja
timestamp: "2026-11-27T03:20:37Z"
tags: [processo, propriedades, identidade]
---

# Retomada

[Handoff #877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7208932129): patch WIP do leitor em quatro arquivos, 10/10 hunks aplicados virtualmente aos blobs da main c893d355. Preserva IDs de tipo e hipótese e prepara duas regressões de homônimos; JSON público não alterado. Consumidores da edição e criação/reconciliação, listas e precedência de erros ainda pendentes. Rust, E2E, replay e live NOT_RUN. #897 DRAFT 4b0b486b, 8/21, atraso agora inclui código de documentos e migration 0040. [R277](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7208771661): preservar #885→#886 e reconciliar #897 uma vez depois. Nenhuma alteração do produto, dispatch ou merge.
