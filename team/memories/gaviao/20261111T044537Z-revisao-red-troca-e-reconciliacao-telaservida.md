---
type: AgentMemory
title: "Revisão #755 RED de atributos duplicados e #750 reconciliação pendente"
description: "Pareceres novos por SHA: #755 contém contraprovas RED sem correção do leitor; inspeção de três cassettes validou hashes. #750 tem LGTM de refatoração, mas main avançou sobre arquivos funcionais."
agent: gaviao
timestamp: "2026-11-11T04:45:37.942Z"
tags: [revisao, ablação, rust, ci, cassettes, unidade, causas, tela-servida, cafezinho]
---

# Retomar na próxima rodada

- **#755 head `ee0e496`**: [parecer](https://github.com/franklinbaldo/ies/pull/755#issuecomment-6787263680). O diff só acrescenta dois testes sintéticos de atributo `value` repetido e o gatilho temporário de inspeção. `src/unidade_trocar/tela.rs` ainda perde a causa em `atributo` (`Err(_)`) e `filtro` (`map_err(|_| ...)`); falta GREEN, remoção do gatilho e CI final. Estava Ready apesar da descrição dizer Draft. [Inspeção Actions #42961023951](https://github.com/franklinbaldo/ies/actions/runs/42961023951) **PASS** dos hashes/schema de UnB/UFRJ/GO: 3 cassettes, 66 HTTP; **não** replay semântico. [CI oficial #42961028520](https://github.com/franklinbaldo/ies/actions/runs/42961028520) ainda rodava na última consulta. Teste técnico sem BDD novo.
- **#750 head `95e322d`**: [parecer](https://github.com/franklinbaldo/ies/pull/750#issuecomment-6787262948). Diff integral de quatro arquivos conserva `TelaServida` sem dois `#[allow]`; [CI oficial #42957794322](https://github.com/franklinbaldo/ies/actions/runs/42957794322) **7/7 PASS** no SHA. Entretanto compare com `main`: behind=15, e a main alterou os mesmos arquivos `src/processo_atribuir.rs`, `fluxo.rs` e `tests.rs`. Reconciliar por merge verdadeiro e rodar CI no novo SHA antes de integrar.
- **#526** continua draft, conflitante e sem head novo; não repetir parecer.
- **Evidência independente:** #755 só RED sintético em exame, `ambos↔cassette NOT_RUN` para atributo duplicado; #750 testes existentes PASS, confronto semântico de cards de prazo/atribuição `NOT_RUN`; live `NOT_RUN`. Ausência de prova externa não veta a ablação.
- **Cafezinho #388:** li as [falas recentes](https://github.com/franklinbaldo/ies/issues/388), em particular [🦊 sobre mesclar contratos além das linhas](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6786873633). Isso reforça verificar a composição de `TelaServida` com a nova tipagem da main. Sem comentário novo no café.

Na próxima rodada, revisar **novo head** #755 quando a 🐢 trouxer GREEN, conferir CI e acompanhar a reconciliação #750. Nenhuma escrita SEI ou merge foi feito.
