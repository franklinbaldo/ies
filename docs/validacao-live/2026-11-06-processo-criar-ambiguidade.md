---
type: SEIObservation
title: Ambiguidade na reconciliação da criação de processos
description: RED sintético, classificação de dois candidatos e validação live pendente da issue 245
operation: processo.criar
date: 2026-11-06
status: not-run
---

# Reconciliação de candidatos ambíguos — issue #245

## Evidência e método

A estrutura da caixa Gerados, do link observado da própria linha e das
propriedades independentes foi documentada anteriormente no
[relatório de reconciliação](2026-11-05-processo-criar-reconciliacao.md),
na decisão da operação e nos testes da issue #245. Esta fatia não extrapola
o cenário sintético para outras instalações.

O teste novo precedeu a mudança de produção.
A primeira compilação [CI 42182686032](https://github.com/franklinbaldo/ies/actions/runs/42182686032)
falhou pela tentativa de ler um campo de diagnóstico ainda não existente
no RED; o teste foi corrigido sem implementação.
O RED executável [CI 42182787186](https://github.com/franklinbaldo/ies/actions/runs/42182787186)
demonstrou **1 teste novo FAIL, 1.679 PASS, 18 ignored preexistentes**:
o fluxo atribuía `efeito_incerto_na_criacao` a dois candidatos
compatíveis independentemente conferidos. O outro novo teste passou:
duas leituras, uma só correspondência, permitem a classificação positiva
somente depois de conferir o segundo candidato.

## Orçamento sintético

| Situação | GET inicial | POST criador | GET Controle posterior | GET de candidatos | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dois candidatos compatíveis | 3 | 1 | 1 | 6 | **11** |
| Dois candidatos, só um compatível | 3 | 1 | 1 | 6 | **11** |

Todos os dados de prova (origem, hash, cookie, NUP e respostas) são
sintéticos nos testes. O POST é injetado e falha sem resposta; não há
requisição real nem retry. A saída ambígua é 3 e distingue os dois NUPs
verificados. A ausência de correspondência não atesta não aplicação.

## Validação ao vivo em treinamento

- Prévia: **NOT_RUN** (0 requisições).
- Um único `--confirmar`: **NOT_RUN** (0 requisições, 0 POST).
- Verificação por leitura: **NOT_RUN** (0 requisições).
- Escritas e resíduos novos: **0**.
- Motivo: nenhum ensaio de segunda instalação foi iniciado nesta fatia;
  a execução segura depende de acesso autenticado de treinamento e da
  sequência prévia → confirmação única → leitura, sem repetir nenhum POST.

O resíduo histórico da **UnB de treinamento**
`23106.001875/2026-93` (ID 51731) permanece como documentado no
relatório original; **não repetir sua criação**.

A issue #245 continua WIP. Falta provar não aplicação com evidência
independente, completar as saídas públicas previstas na issue e validar
ao vivo em uma segunda instalação. Não declarar a capacidade completa.

## Continuação 🦉 — recusa expressa de criação (06/11/2026)

Observação anterior: `txaInfraValidacao` publica a recusa do próprio SEI;
HTTP 403 isolado não comprova por si só que o cadastro não foi aplicado.
RED test-first [CI 42187792547](https://github.com/franklinbaldo/ies/actions/runs/42187792547):
**2 FAIL, 1.679 PASS, 18 ignorados preexistentes**: um teste novo
de diagnóstico público e uma nova asserção no teste de transporte da recusa.
O desfecho `nao_aplicada` é publicado somente para recusa expressa da
Infra, com saída 1, causa do POST e nova prévia exigida antes de qualquer
nova confirmação. O erro HTTP 403 mantém a causa e não prova ausência.

**Orçamento sintético exato:** recusa Infra: 3 GET + 1 POST = 4 pedidos;
HTTP 403: 3 GET + 1 POST = 4 pedidos. **Sem GET posterior nem retry**.
Não foi introduzido novo POST ao vivo.

**LIVE nesta execução:** prévia NOT_RUN (0), confirmação NOT_RUN (0),
releitura NOT_RUN (0), zero requisições reais, zero escritas e zero
resíduos novos. A escrita UnB histórica permanece registrada sem repetição.
**Issue WIP:** faltam prova não aplicada para outros cenários com evidência
independente, homologação segura em segunda instalação de treinamento e
revisão/integração da fatia.


## Continuação 🦉 — HTTP 403 após POST não comprova não aplicação (06/11/2026)

**Observação anterior ao RED:** o cenário HTTP 403 do transporte injetado
já estava representado em `tests_candidatos.rs` e havia parecer do 🦅
[para o head 00dffe9](https://github.com/franklinbaldo/ies/pull/315#issuecomment-6680875093):
um 403 isolado pode ser emitido por intermediário, logo não prova
ausência de criação. A correção não se baseou em novo ensaio real.

**RED observado:** [CI 42195250905](https://github.com/franklinbaldo/ies/actions/runs/42195250905),
commit de testes `de8fa69`: **1 FAIL, 1.688 PASS, 22 ignored
preexistentes**. Falhou precisamente o teste de HTTP 403:
esperava saída 3 e recebeu 1. Nenhum teste foi desativado.

**GREEN:** correção tipada separa validação expressa da Infra (saída 1,
`nao_aplicada`) de HTTP 4xx sem prova (saída 3,
`efeito_incerto`); mantém subcódigo, status e mensagem saneada,
sem releitura que possa atribuir processo concorrente e sem retry.
O CI do head final deve ser conferido na PR antes de integrar.

**Orçamento sintético exato:** recusa Infra e HTTP 403: **3 GET +
1 POST criador, 4 pedidos**, zero GET posterior, zero segundo POST.

| Etapa de treinamento nesta fatia | Estado | Requisições reais | Resíduo novo |
| --- | --- | ---: | --- |
| Prévia autenticada | NOT_RUN | 0 | Nenhum |
| Um único `--confirmar` | NOT_RUN | 0 | Nenhum |
| Verificação por leitura | NOT_RUN | 0 | Nenhum |
| Replay público (gate CI) | A conferir no head final | 0 | Nenhum |

**Total live: 0 requisições reais, 0 escritas, 0 resíduos novos.**
Não repetir o POST da UnB que criou `23106.001875/2026-93`.
Pendente: prova adicional de não aplicação onde houver, teste
autenticado em outra instalação **de treinamento** com prévia →
confirmação única → verificação por leitura e registro do resíduo.
A issue continua WIP.

## Continuação 🦉 — preservar observações após falha verificadora (06/11/2026)

Uma leitura posterior do Controle pode identificar candidatos novos, mesmo
que o GET seguinte da tela, árvore ou propriedades falhe. Antes, o
relatório apresentava `candidatos_observados_na_caixa: null`, perdendo
NUPs efetivamente vistos. A observação pré-RED veio da estrutura das
fotografias de Gerados e do teste sintético de leitura já existente.

O teste novo injeta falha de rede somente na leitura independente do
candidato, depois de observar uma linha Gerados. Orçamento exato:
**3 GET iniciais + 1 POST criador + 1 GET Controle + 1 GET falho = 6**.
Mantém saída 3, causa do POST, causa da verificação e o NUP observado;
não confirma aplicação, não repete a escrita e não divulga URL assinada.

**RED:** [CI 42202730391](https://github.com/franklinbaldo/ies/actions/runs/42202730391),
**1 FAIL, 1.695 PASS, 22 ignorados preexistentes**, falha na comparação
`None` versus NUP observado. **GREEN:** conferir no CI do head final.
**LIVE:** prévia NOT_RUN (0), confirmar NOT_RUN (0), releitura NOT_RUN (0);
**0 requisições reais, 0 escritas, nenhum resíduo novo**. Não repetir
`23106.001875/2026-93` (UnB de treinamento). A issue segue WIP.


## Continuação 🦉 — falha após `Location` válido (06/11/2026)

**Observação pré-RED:** o fluxo Web documentado devolve um redirecionamento
após o POST criador, mas só a leitura independente da tela e árvore
confere NUP e ID. Um status de erro no GET de verificação é fato
distinto de recusa do POST: não comprova não aplicação.

**RED observado:** [CI 42210005212](https://github.com/franklinbaldo/ies/actions/runs/42210005212),
**2 FAIL, 1.705 PASS, 22 ignored preexistentes**. O HTTP 503 no primeiro
GET verificador perdia `status_http`; a falha de rede perdia o
subcódigo `falha_de_rede`. Os testes permanecem ativos.

**Implementação:** a verificação após `Location` preserva a causa do
GET/DOM em variante tipada, mas continua **saída 3** até comprovar a
criação. Sem retry, sem novo POST, sem sucesso por redirecionamento.

**Orçamento sintético exato:** cada teste usa 3 GET iniciais + 1 POST
criador + 1 GET verificador que falha = **5 pedidos** (4 GET, 1 POST),
zero requisições posteriores.

**Live em treinamento nesta fatia:** prévia **NOT_RUN** (0 requisições);
confirmação **NOT_RUN** (0); verificação por leitura **NOT_RUN** (0).
Total **0 requisições reais**, **0 escritas**, **0 resíduos novos**.
Não repetir o POST histórico da UnB que criou o NUP
`23106.001875/2026-93` (id 51731). Segunda instalação ainda
pendente; confirmar somente se for possível cumprir toda a sequência
prévia → confirmação única → verificação por leitura → registro do
resíduo. Issue #245 continua WIP.
