---
type: AgentMemory
title: "Ablação #717 integrada; BDD-381-04 sob responsabilidade da Raposa"
description: "Verifiquei na main os erros tipados de índice e marcador após #738, encerrei #717; PR #737 ocupa a prévia processual, sem branch concorrente."
agent: polvo
timestamp: "2026-11-11T01:52:44Z"
tags: [ablacao, erros-tipados, documentos, blocos, bdd, anti-ies]
---

# Para retomar

- **Fato:** PR [#738](https://github.com/franklinbaldo/ies/pull/738) integrada com [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42940623563). Verifiquei `ErroAberturaDoIndice` em `src/indice_local/abertura.rs` e `ErroMarcadorDaSessao` em `src/sessao_login/gravacao.rs`; a família explicitada em [#717](https://github.com/franklinbaldo/ies/issues/717) foi registrada como concluída e a issue fechada, [comentário](https://github.com/franklinbaldo/ies/issues/717#issuecomment-6784652843). A conversão do JSON em erro de marcador retém categoria e posição, enquanto transporte retém fonte.
- **Coordenação:** [PR #708](https://github.com/franklinbaldo/ies/pull/708) já integrada; **BDD-381-04** foi assumido pela 🦊 na [PR #737](https://github.com/franklinbaldo/ies/pull/737). No head `fbd10506`, CI rápido PASS, [oficial #42941958023](https://github.com/franklinbaldo/ies/actions/runs/42941958023) ainda `in_progress`; conferir o head antes de promover. Não alterar a branch da Raposa nem assumir BDD-381-05 antes da integração.
- **Prioridade nova do AGENTS.md:** campanha de antipatterns prevalece sobre abertura de novas capacidades. Depois de #717, triagem seguinte #718 e #719; uma alteração observável da CLI requer cenário canônico do 🐘. A #737 preserva a separação de prova sintética, bilateral com cassette PEN1 e live.
- **Provas desta rodada:** nenhuma nova CLI↔anti-IES↔SQLite, comparação com cassette ou live executada pelo 🐙; foram conferidos código da main, diff da #737 e CI de PRs existentes. Sem SEI externo, credenciais ou escrita.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa sobre POST de pesquisa, diferente de POST mutador, ajuda a não contar envelope HTTP como efeito.
