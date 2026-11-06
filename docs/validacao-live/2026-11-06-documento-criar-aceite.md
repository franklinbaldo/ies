---
type: Guide
title: Regressão offline do aceite de criação de documento
description: Evidência RED, cenário concorrente e validação live pendente na issue 246
---

# Documento criar — aceite após POST e efeito incerto

Issue [#246](https://github.com/franklinbaldo/ies/issues/246);
[PR #248](https://github.com/franklinbaldo/ies/pull/248).

**Observação anterior ao RED:** o parecer 🦅 do head `76912e7`
identificou HTML 200 como resposta inconclusiva e perda da causa de
falhas HTTP após POST único. Um agente diferente da mesma conta pode
criar documento de tipo idêntico entre as duas fotos da árvore.

**RED:** testes adversariais sintéticos publicados antes da correção,
commit `397f9ed` e [CI 42229169889](https://github.com/franklinbaldo/ies/actions/runs/42229169889):
**3 FAIL, 1.753 PASS, 23 ignorados**. As três falhas mostraram:
aceite falso do HTML 200; perda de HTTP 403 e 503. Sem requisição real.

**Roteiro de teste:** 1 POST sintético + 1 GET de releitura
independente por cenário, sem retry. Aceite positivo só com 302
para o controlador original e `id_documento` igual ao novo nó
observado. Falha posterior ao envio preserva `causa_envio`,
`status_http` e `mensagem_do_servidor` saneada, com saída 3.

**LIVE nesta execução:** prévia **NOT_RUN**, confirmação **NOT_RUN**,
verificação independente **NOT_RUN**; **0 requisições reais ao SEI,
0 POST reais, 0 escritas, nenhum resíduo**.
Não foi executada a sequência autenticada em treinamento. A resposta
real da instalação ainda precisa ser observada antes de admitir
uma nova forma de aceite. Se a prévia ou retorno forem inconclusivos,
não repetir `--confirmar`.

**Falta:** CI do head definitivo; revisão 🦅; reconciliação do branch
com a main; ensaio live em TREINAMENTO com prévia segura → um único
`--confirmar` → leitura independente e registro do número SEI/resíduo.
