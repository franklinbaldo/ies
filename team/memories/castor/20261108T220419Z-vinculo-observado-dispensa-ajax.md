---
type: AgentMemory
title: "Vínculo já observado dispensa consulta complementar"
description: "A prévia da CLI real agora tem regressão para relação preexistente, com estado SQLite preservado e nenhuma busca AJAX; CI completo 7/7 no head da PR #491."
agent: castor
timestamp: "2026-11-08T22:04:19Z"
tags: [bdd, rust, anti-ies, sqlite, cli, ci]
---

# Encaixe da próxima rodada

**Fato:** a [PR #491](https://github.com/franklinbaldo/ies/pull/491) recebeu merge verdadeiro da `main@6fc68a2`, preservando só as sete memórias recém-adicionadas e o esquema v12→v13→v14→v15. O novo teste `bdd_258_01_relacao_preexistente_nao_faz_consulta_ajax_nem_escrita` executa a CLI real por TLS loopback contra anti-IES/SQLite e exige `sem_efeito=true`, nenhuma consulta AJAX redundante nem mutação do vínculo. [CI #42528039842](https://github.com/franklinbaldo/ies/actions/runs/42528039842): **7/7 PASS** no head `fc3520d7`. [Handoff #258](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6727752128).

**Retomada:** revisão/integração 🦅/🐘/🦡 ainda pendente; a `main` voltou a avançar em outras frentes. BDD-258-02/03/04 seguem pendentes. A prova é **sintética**; cards PEN1 tiveram integridade verificada anteriormente, mas confronto semântico independente e validação live não foram executados. Não alegar RED funcional neste teste adicional; o primeiro CI rápido falhou por formatação, corrigida.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726392005), a imagem dos pedais e da fita saturada reforçou uma ideia de ofício: resultados idênticos podem esconder histórias distintas. É inspiração ficcional para pedir releitura independente, **não evidência do SEI**.
