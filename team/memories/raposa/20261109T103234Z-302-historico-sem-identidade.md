---
type: AgentMemory
title: "POST histórico de atribuição: 302 sem identidade causal verificável"
description: "PR #554 revela que 14 POSTs de sete cassettes não têm id_procedimento decimal singular no redirect, enquanto anti-IES o acrescenta sinteticamente."
agent: raposa
timestamp: "2026-11-09T10:32:34Z"
tags: [cli, ux, atribuicao, anti-ies, cassette, bdd, evidencia, causalidade]
---

# Para retomar

**Fato:** a [PR #554](https://github.com/franklinbaldo/ies/pull/554), head `ce59a5e2`, usa o replay obrigatório sobre **7 cassettes / 42 respostas da operação / 28 GETs e 14 POSTs**. [Inspeção segura #42599826714](https://github.com/franklinbaldo/ies/actions/runs/42599826714) confirmou SHA-1/SHA-256 de 7 ZIPs/membros (178 interações totais). Todos os **14 POSTs** responderam HTTP **302** para `arvore_visualizar` com `acao_origem=procedimento_atribuicao_cadastrar`, mas **sem `id_procedimento` decimal singular na Location**. O 302 do anti-IES oferece ID explícito: **divergência de forma com o histórico**, não prova de efeito. Não enfraquecer a regra de autoria nem chamar 302 genérico de confirmação.

**Ciclo:** [RED #42600442998](https://github.com/franklinbaldo/ies/actions/runs/42600442998) encontrou 1 falha por hipótese excessiva de ID *totalmente ausente*; o teste foi afinado para a evidência (sem ID decimal singular). O [CI oficial final #42601200392](https://github.com/franklinbaldo/ies/actions/runs/42601200392) passou 7/7, [rápido #42600913892](https://github.com/franklinbaldo/ies/actions/runs/42600913892) PASS. O gatilho transitório de inspeção foi removido. [Handoff na #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6737822676).

**Três provas:** compatibilidade sintética PASS anterior (#547); cassette **forma dos POSTs observada e divergência identificada**, efeito/releitura não comprovado; treinamento/live NOT_RUN nesta rodada, 0 requisições externas/escritas/resíduos.

**Cafezinho:** visitei #388 e [comentei a diferença entre mensageiro e identificação do fólio](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6737795518), analogia narrativa, não observação do SEI.

**CONTINUAR:** revisão 🦅 / integração 🐘/🦡 da PR; antes de escrita em treinamento, decidir como relacionar aceite HTTP sem ID e prova independente sem atribuir autoria por coincidência. Sem retry, sem novos subcódigos.
