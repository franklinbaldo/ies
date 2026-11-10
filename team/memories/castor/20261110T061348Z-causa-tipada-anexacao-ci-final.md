---
type: AgentMemory
title: "BDD-270-02: causa tipada sem subcódigo e reconciliação final"
description: "PR #636 conciliou a main, trocou o novo subcódigo manual por etapa Rust serializada via Serde e passou CI oficial 7/7 no head bb3c2e4; integração pendente."
agent: castor
timestamp: "2026-11-10T06:13:48Z"
tags: [ies, anti-ies, anexacao, bdd, rust, serde, erros-tipados, sqlite, ci]
---

# Lembrar na próxima rodada

**Fato:** a [PR #636](https://github.com/franklinbaldo/ies/pull/636) foi reconciliada com `main@7c7be970` por merge verdadeiro de dois pais, head `bb3c2e4628eb`, sem rebase nem force-push. A única sobreposição de arquivos era `tests/cli_processo_anexar.rs`: preservei o subcódigo legado atual `vazio` da main para número inválido e a confirmação sem sessão da fatia. Migração de anexação continua v32, após marcadores v31.

**Correção tipada:** após o parecer 🦡/🐘, removi a nova tradução `FalhaAnexar::EfeitoIncerto => "efeito_incerto"` em `fn subcodigo()`. O erro agora carrega `EtapaNaoComprovada` (resposta do POST, lista após POST ou árvore do principal), publicada diretamente por Serde em `causa`. A categoria de decisão `codigo=efeito_incerto` conserva saída 3; o JSON diz `escrita.envio_tentado=true` e `aplicada=null`, sem alegar sucesso. A saída 3 não publica `subcodigo`; os legados permanecem em `subcodigo_legado`. O teste CLI comprova os campos novos, POST único e releitura.

**Validação:** [CI oficial #42766644579](https://github.com/franklinbaldo/ies/actions/runs/42766644579) **7/7 PASS** e [rápido #42766578409](https://github.com/franklinbaldo/ies/actions/runs/42766578409) PASS no mesmo head; `behind=0` na conferência. IES ↔ anti-IES ↔ SQLite: **PASS sintético**; integridade UnB/GO, 102 interações, **PASS anterior**; confronto semântico independente das transições: **NOT_RUN**; treinamento/live: **NOT_RUN**. Nenhuma escrita ao SEI, produção ou credencial. O campo `causa` identifica a **etapa** que não foi comprovada, não é a causa de transporte original quando esta existe: ponto possível para melhoria posterior sem bloquear a integração.

**Cafezinho:** a [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6760482794) associou o cuidado da 🦊 com presença versus disponibilidade à distinção entre envio tentado e efeito comprovado. Inspiração metodológica, não evidência externa.

**CONTINUAR:** aguardar revisão 🦅 do head tipado e integração 🐘/🦡, checando `behind` e CI no SHA. Depois da integração, conferir BDD-270-01/03 na #270 antes de assumir nova frente, sem duplicar trabalho.
