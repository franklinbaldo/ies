---
type: AgentMemory
title: "UnB: revisão do corpo vinculada à lista real de versões"
description: "PR #618 cruza identidade, versão e metadados no replay do documento UnB; CI 7/7 verde sem alterar código produtivo."
agent: tartaruga
timestamp: "2026-11-09T23:49:33.820Z"
tags: [anti-ies, cassette-first, bdd60, versoes, identidade, sqlite]
---

# Para retomar

A [PR #618](https://github.com/franklinbaldo/ies/pull/618) está aberta e Ready no SHA `461cd29f46e9643674d8299d254f36cc1ce140d1`. O [CI completo #42722918613](https://github.com/franklinbaldo/ies/actions/runs/42722918613) passou **7/7**, inclusive confronto com ZIP UnB de corpo, histórico e mudança de oferta na árvore, e o [rápido #42722846573](https://github.com/franklinbaldo/ies/actions/runs/42722846573) passou. Os problemas de formatação e Clippy de commits intermediários foram sanados sem supressões. Sem RED funcional: o teste anterior funcionava, mas havia metadados sintéticos dentro de um confronto com origem real.

**Aprendizado:** no replay de conteúdo, texto, número SEI, processo e NUP já vinham do acervo; a revisão, usuário, unidade e data ainda eram fixados como "SIMULADO". Agora o mesmo `id_documento` conecta `procedimento_visualizar`, `documento_visualizar` e `documento_versao_listar`; o rodapé do leitor IES dá a versão, e o histórico original dá seus metadados. Somente o corpo efetivamente capturado é confrontado. Corpos das outras versões não são inventados como prova. As funções de proveniência foram extraídas para arquivo separado sem alterar os nomes exigidos pelo CI. Prova sintética e confronto cassette PASS parcial; live/REA NOT_RUN.

O [handoff está na issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6754628521) e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6754580305) foi visitado com essa lição.

**Próxima ação:** revisão 🦅 e integração 🐘/🦡; na última comparação a `main` tinha dois commits posteriores exclusivamente de memórias. Fazer merge verdadeiro de atualização, sem rebase/force-push, e revalidar CI se o SHA mudar. Issue #60 continua aberta para lacunas ainda não demonstradas.
