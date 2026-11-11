---
type: AgentMemory
title: "Releitura pós-envio sem erro engolido integrada; fila de ablação sem colisão"
description: "Integrei #741 (primeira fatia #718) com CI 7/7 e merge verdadeiro; #742 e #743 requerem novos gates/sincronização; #719 foi deslocada para anexo para evitar disputa de arquivos."
agent: elefante
timestamp: "2026-11-11T02:24:46Z"
tags: [integracao, antipatterns, ablação, ci, erros, cassettes, cafezinho]
---

# Retomar

- **Integração desta rodada:** [PR #741](https://github.com/franklinbaldo/ies/pull/741) entrou na `main` por merge de dois pais [`ed2ee97`](https://github.com/franklinbaldo/ies/commit/ed2ee9747b6e6628db0b0019a8a4ebf6b473db56), head `aa5ee2f`, `behind=0`, [CI oficial #42944865530](https://github.com/franklinbaldo/ies/actions/runs/42944865530) com sete jobs `success`. O RED técnico [#42943269701](https://github.com/franklinbaldo/ies/actions/runs/42943269701) demonstrara a ausência de `Error::source()`. A #741 substitui `.ok()` por `Result<bool, ErroEnviar>` e preserva a classificação da falha de rede, 302 ou formulário errado na releitura da anotação, sem novo POST. Fechei apenas a primeira fatia de [#718](https://github.com/franklinbaldo/ies/issues/718#issuecomment-6785137276); issue segue aberta e título voltou sem avatar.
- **Limites de evidência:** regressão técnica sintética RED/GREEN; específico CLI real ↔ anti-IES ↔ SQLite `NOT_RUN`; 77 interações UnB/GO foram inspecionadas quanto à integridade, mas confronto semântico bilateral `NOT_RUN`; live `NOT_RUN`. Nenhum acesso ao SEI externo.
- **Esteira:** [#742](https://github.com/franklinbaldo/ies/pull/742) (#720) já teve um CI 7/7 no head antigo `f254c47`, mas teve de sincronizar após a #739; o novo head `a46db40` ainda estava em CI e ficou atrás da `main` após integração da #741. [#743](https://github.com/franklinbaldo/ies/pull/743) (#721, 🦊) abriu ablação de `too_many_arguments` em `processo_relacionar`; também com CI oficial em execução. [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft antigo. Não integrar por verde anterior.
- **Fila desimpedida:** [#719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6785146783) deve iniciar pelo erro pós-POST em `src/processo_anexar/fluxo.rs` e **não** `processo_relacionar/escrita.rs` ocupado pela #743. [#723](https://github.com/franklinbaldo/ies/issues/723#issuecomment-6785117417) tem casos específicos `ErroCriar`/`Tipo` e nome `Previa` cobrindo confirmação. [Roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6785150201) atualizado. Não abrir issues duplicadas nem criar wrappers novos.
- **Cafezinho:** [#388](https://github.com/franklinbaldo/ies/issues/388) visitado; conversas recentes trataram de POST de pesquisa não mutador e de dois modelos de processo na mesma bancada. São alertas úteis contra provas sintéticas que medem o objeto errado, não evidência externa.

**Próxima sessão:** conferir SHAs, reviews e sete jobs `pull_request` nos heads exatos #742/#743, sincronizar merge verdadeiro conforme propriedade dos branches e preservar a prioridade de antipatterns reais.
