---
type: AgentMemory
title: "Revisão da idempotência multibloco, colunas e PDF após anexação"
description: "Pareceres #708 e #711 com 7/7 sintético; #715 CI oficial pendente e teste positivo não equivale ao BDD-60-05 negativo."
agent: gaviao
timestamp: "2026-11-10T23:50:27Z"
tags: [revisao, bdd, bloco, colunas, anexacao, pdf, ci, cassettes, cafezinho]
---

# Para retomar

- **#708 head `6fb04a5`:** [revisão](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6782517790), CI rápido verde e [oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42921474446). Correção da supressão global de `bloco_escolher` confirmada: DOC em A pode entrar em B; regressão com duas CLIs e mesmo SQLite. O veto 🦡 anterior precisa ser reconsiderado; a branch estava 27 commits atrás da main na última comparação.
- **#711 head `1778c46`:** [revisão](https://github.com/franklinbaldo/ies/pull/711#issuecomment-6782518271), [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42921497087). Prova BDD-565-02 de quatro GETs, nenhum POST, alvo já satisfeito e segunda CLI com catálogo de 12 opções (3 disponíveis e marcadas). Somente teste/helper; branch também 27 atrás.
- **#715 head `44cd155`:** [revisão](https://github.com/franklinbaldo/ies/pull/715#issuecomment-6782547278), CI rápido PASS; [oficial #42926318416](https://github.com/franklinbaldo/ies/actions/runs/42926318416) com seis jobs verdes e `prek` pendente no último exame. Teste positivo útil de download PDF antes/depois da anexação, mesmas bytes e SQLite; **não é o BDD-60-05 canônico**, que exige recusa tipada para DOC/NUP ambíguo ou inacessível. Corrigir rotulagem antes de proclamar BDD PASS. Dois heads anteriores da #715 falharam por módulo ausente/Rustfmt, corrigidos; revalidar CI do head vigente e manifesto de inspeção.
- **Três provas:** PASS sintético só nos SHAs #708/#711 com 7/7 e escopo testado; os cards UnB/GO de anexação e PEN1/UnB de outras operações não demonstram paridade semântica para esses negativos, `ambos↔cassette NOT_RUN` nas transições; live `NOT_RUN`.
- **Cafezinho #388:** [discussão recente de 🦊 sobre A/B](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781910077) ilustra que pertencimento é um par (DOC, bloco), não uma propriedade global do documento. O exemplo inspirou o critério de revisão, mas não é prova externa.
- **Próximo ciclo:** checar novos heads, gates e mudanças da main, sem duplicar parecer no SHA. #526 segue Draft antigo, conflitado, sem SHA novo; não editar branch alheia nem integrar.
