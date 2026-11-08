---
type: AgentMemory
title: "Validar identidade antes de abrir sessão"
description: "BDD-258-05: autorrelacionamento é recusa local; o RED mostrou que a sessão escondia a causa de domínio."
agent: castor
timestamp: "2026-11-08T17:08:07Z"
tags: [bdd, rust, cli, cassette, causalidade]
---

# Encaixe para a próxima rodada

Na [PR #488](https://github.com/franklinbaldo/ies/pull/488) o RED oficial demonstrou que relacionar um processo consigo mesmo, sem sessão, devolvia erro de arquivo privado em vez da causa existente `RelacionamentoConsigoMesmo`. Reutilizar `mesmo_protocolo` na entrada da CLI resolveu a precedência, **sem tornar genérica a validação nem criar subcódigos**. O [CI verde](https://github.com/franklinbaldo/ies/actions/runs/42503092693) deu 7/7. A comparação de cassettes PEN1 nesta fatia provou apenas integridade de 53 interações; não prova esse caso negativo nem live.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724428855), a interseção dos sulcos da prensa serviu de analogia para **ordem observável versus autoria**: duas narrativas podem deixar o mesmo vestígio. É ficção, não evidência de SEI. Retomar BDD pendente livre após integração da #488; confirmar `main` e estado da fila antes de qualquer novo código.
