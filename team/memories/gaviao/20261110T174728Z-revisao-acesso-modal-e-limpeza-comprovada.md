---
type: AgentMemory
title: "Revisões da modal de senha e da limpeza de anotação antes do envio"
description: "Pareceres #688 e #689: recusa de acesso preservada, BDD-581-01 comprovado em duas CLIs, ambos com CI 7/7; integração depende de sincronização da main."
agent: gaviao
timestamp: "2026-11-10T17:47:28Z"
tags: [revisao, bdd, acesso, anotacao, envio, anti-ies, cassettes, ci, cafezinho]
---

# Retomar

**Fato #688:** [parecer head `49c4ef5`](https://github.com/franklinbaldo/ies/pull/688#issuecomment-6774780751), CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42866392895). BDD-633-02 parcial: a tela de `usuario_validar_acesso` não é link velho; a CLI real preserva recusa, saída 1, dois GET, zero POST, SQLite v36 e sem divulgar conteúdo. Falta mensagem específica de credenciais 0600 ausentes; BDD-633-01/03 não contemplados. Comparação anterior à memória: `behind=1`. Hashes GO (48 interações) não são confronto semântico.

**Fato #689:** [parecer head `2b15bb5`](https://github.com/franklinbaldo/ies/pull/689#issuecomment-6774781514), CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42867905892), CI rápido verde. BDD-581-01 sintético executado: POST único de limpeza comprovada por GET, envio inválido **antes** do POST, saída 1, `anotacao_depois:false`, segunda CLI na unidade de origem e zero POST de envio. A variante tipada conserva causa e categoria pública preexistente; `enviar()` separa erros pós-POST. RED funcional `null != false` documentado; Clippy que reprovou no head intermediário foi corrigido. `behind=0` antes desta memória. BDD-581-02 continua.

**Três provas:** PASS sintético para as fatias; cassettes UnB/GO apenas hashes/estrutura (sem confronto bilateral); live desta rodada NOT_RUN. Não acessar SEI nem credenciais.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente a [nota da 🦊 sobre a #689](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6774341765): o segundo ato que falha não desfaz um primeiro ato já comprovado. Lição útil para revisar resultados parciais tipados, não evidência do SEI real.

**Próxima rodada:** reconferir SHAs e atrás da `main` após esta memória; revalidar os sete gates no SHA final antes de integrar. Não repetir parecer para mesmo head. #526 segue Draft/conflitante sem head novo. Não houve implementação, merge ou uso de produção.
