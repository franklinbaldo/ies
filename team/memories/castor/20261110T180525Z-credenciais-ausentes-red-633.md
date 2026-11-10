---
type: AgentMemory
title: "BDD-633-02: RED real para a mensagem de credenciais ausentes"
description: "O CI oficial mostrou RED da CLI real quando a recusa de acesso não explicava a ausência de IES_CREDENCIAIS_FILE; correção textual tipada enviada na PR #688, GREEN ainda a validar."
agent: castor
timestamp: "2026-11-10T18:05:25Z"
tags: [ies, rust, bdd, anti-ies, sqlite, credenciais, red-green]
---

# Próximo encaixe

**Fato:** no [CI #42870288566](https://github.com/franklinbaldo/ies/actions/runs/42870288566), o teste BDD-633-02 da **CLI Rust real ↔ TLS local ↔ anti-IES ↔ SQLite v36** falhou como esperado (RED): a saída 1 mencionava a exigência de senha, mas não a falta do arquivo privado. Os outros três testes da bancada passaram. Corrigi somente a mensagem da mesma variante tipada quando `IES_CREDENCIAIS_FILE` está ausente, sem novo subcódigo nem tentativa de senha; após falha Rustfmt do CI rápido, ajustei a linha no head da [PR #688](https://github.com/franklinbaldo/ies/pull/688). **GREEN ainda não estava validado** ao registrar esta memória.

**Evidência:** os três cards GO (48 interações) têm inspeção estrutural PASS, mas não trazem a ação da modal; a modal do SQLite é hipótese sintética. Confronto semântico bilateral e treinamento/live **NOT_RUN**. BDD-633-01/03 continuam pendentes, sem POST externo.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a conversa sobre não confundir ausência de observação com prova negativa. Diagnóstico de configuração ausente é contexto da falha, não prova de que a validação do servidor foi tentada.

**CONTINUAR:** incorporar esta memória à branch por merge verdadeiro, executar os sete gates oficiais no head final; registrar GREEN somente após execução. Handoff para revisão e integração 🦅/🐘/🦡.
