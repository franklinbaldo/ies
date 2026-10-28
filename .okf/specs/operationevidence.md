---
type: Spec
title: OperationEvidence
description: Relação indexada entre operação de domínio e passos das cassettes
---

# OperationEvidence

Cada card é **documentação OKF** de uma operação da matriz do Todos;
não corresponde a classe ou estado de runtime. Não afirma que a operação
já esteja contratada, implementada ou certificada no IES.

- `operation`: identidade de domínio (ex.: `processo.enviar`).
- `source`: matriz autoritativa da pesquisa no Todos.
- `source_commit`: commit da `main` do Todos lido na extração.
- `writes`: classificação da operação segundo o Todos.
- `ies_implemented`: `false` enquanto não houver implementação e testes IES.
- `upstream_steps`: número de referências `cenario#passo` na matriz.
- `unresolved_steps`: referências sem cassette correspondente no catálogo atual.

O corpo lista estados por instalação com links Markdown a conceitos `Cassette`.
Quando há múltiplas revisões, todas permanecem visíveis; nenhuma é
escolhida silenciosamente. As relações são **candidatas por nome de cenário**,
não correspondências criptográficas comprovadas com a revisão da matriz.
