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
GET/DOM em erro tipado, mas continua **saída 3** até comprovar a
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

## Continuação 🦉 — HTTP do GET de reconciliação (06/11/2026)

A evidência de base é a distinção já tipada entre falha de rede e
HTTP 5xx no transporte, sem alegar uma nova ocorrência real no SEI.
RED no [CI 42217861035](https://github.com/franklinbaldo/ies/actions/runs/42217861035):
**1 FAIL novo, 1.708 PASS e 22 ignored preexistentes**, porque o POST
sem resposta tinha `status_http: null`, embora o GET posterior houvesse
observado HTTP 503. A correção preserva o status da releitura e,
quando os dois estágios têm status distintos, publica-os separadamente.

Nos testes sintéticos, cada execução consome **3 GET prévios + 1 POST
criador + 1 GET posterior = 5 pedidos**, um único POST, zero retry.
Live nesta fatia: prévia **NOT_RUN (0)**; confirmação **NOT_RUN (0)**;
releitura **NOT_RUN (0)**; total **0 requisições reais**, **0 escritas**
e **0 resíduos novos**. O histórico UnB `23106.001875/2026-93`
não será repetido. Segunda instalação de treinamento e sequência
segura prévia → confirmação única → leitura seguem pendentes.


## Continuação 🦉 — coincidência comprovada antes de falha posterior (06/11/2026)

**Observação pré-RED:** o leitor de Gerados e das propriedades já
observados para #245 permite obter prova completa de um candidato e
falhar em outro. O caso foi reproduzido apenas por transporte e HTML
**sintéticos**, sem nova observação live.

**RED comportamental:** [CI 42226093955](https://github.com/franklinbaldo/ies/actions/runs/42226093955),
**1 FAIL / 1.711 PASS / 22 ignored preexistentes**. A verificação do
segundo candidato falhou; a primeira correspondência já comprovada
não era publicada (`None` em vez do NUP).

**Critério e orçamento de bancada:** 3 GET anteriores, **1 POST**
sem resposta, 1 GET posterior do Controle, 3 GET de verificação do
primeiro NUP e 1 GET falho do segundo = **9 pedidos**. A saída
continua 3; não se afirma autoria, unicidade ou ausência de efeito.
O JSON apresenta as coincidências parciais separadas do desfecho
de ambiguidade completa. Nenhuma requisição é repetida.

**LIVE nesta execução:** prévia **NOT_RUN (0)**; único
`--confirmar` **NOT_RUN (0)**; releitura **NOT_RUN (0)**; total
**0 requisições reais, 0 POSTs reais, 0 escritas e 0 resíduos novos**.
O NUP histórico da UnB treinamento não recebeu novo POST.
Falta ensaio prévia → confirmação única → leitura, com registro
de resíduo, numa segunda instalação de treinamento. Issue #245 WIP.


## Continuação 🦉 — Location válido sem propriedades conferidas (06/11/2026)

**Observação pré-RED:** o fluxo Web publica a árvore e nela o link
de propriedades. Os leitores já existentes provam ID/NUP, tipo,
especificação e nível numa tela independente. A árvore sozinha só
prova identidade, não o conteúdo efetivo da criação.

**RED de transporte sintético:** [CI 42234422953](https://github.com/franklinbaldo/ies/actions/runs/42234422953),
**1 FAIL, 1.718 PASS, 22 ignorados preexistentes**, porque o fluxo
não consumiu a resposta sintética das propriedades. O novo teste
exige recusa de confirmação com saída 3 quando tipo/especificação
lidos não correspondem ao pedido. O caso de sucesso usa um GET
adicional de propriedades, sempre sem segundo POST.

**Orçamento exato da regressão:** 3 GET antes + **1 POST** criador
+ 3 GET verificadores = **7 pedidos** (6 GET e 1 POST).
O link de propriedades é observado na própria árvore; nenhum
redirecionamento seguido, nenhum retry, nenhum JS executado.

**LIVE nesta fatia:** prévia **NOT_RUN (0 req.)**; `--confirmar`
**NOT_RUN (0 req.)**; verificação **NOT_RUN (0 req.)**.
Total **0 requisições reais**, **0 escritas**, **0 resíduos novos**.
Não repetir a criação histórica UnB `23106.001875/2026-93`.
Permanece pendente uma nova sequência segura noutra instalação de
treinamento, com resíduo documentado; issue #245 continua WIP.

## 🦉 RED do nível de acesso na reconciliação (06/11/2026)

**Observação anterior:** leitor de propriedades Web e fixtures sintéticas
`tela_das_propriedades`/`tela_restrita`, derivadas da estrutura
observada na UnB 4.0.9; propriedade `rdoNivelAcesso` já publicada.
A verificação independente após `Location` legítimo conferia o nível,
mas a reconciliação do POST sem resposta não o comparava.

**RED test-first:** [CI 42243502996](https://github.com/franklinbaldo/ies/actions/runs/42243502996),
**1 FAIL novo, 1.724 PASS, 22 ignored preexistentes**. Com 3 GET
pré-POST + 1 POST sem resposta + 1 GET Controle + 3 GET do candidato,
o IES anunciava `confirmado: true` e NUP de um candidato restrito,
apesar de o pedido ser público. A falha do teste ocorreu após os
8 pedidos sintéticos previstos; não houve SEI real.

**Correção:** a leitura de propriedades agora alimenta o classificador
com o nível observado. Uma correspondência completa exige nível exato,
e as coincidências parciais preservadas após falha de outro candidato
seguem a mesma exigência. Sem novos pedidos; zero retry. Nível
divergente não prova não aplicação.

**Live desta fatia:** NOT_RUN, **0 requisições reais, 0 POST, 0 escrita e
0 resíduos**. A criação histórica da UnB não deve ser repetida.
O ensaio autorizado numa segunda instalação de treinamento continua
pendente (prévia → um único `--confirmar` → releitura e registro do resíduo).

## 🦉 Continuação — interessados explícitos e criação concorrente (06/11)

**Evidência anterior ao RED:** `selInteressadosProcedimento` na
tela de propriedades sintética baseada na UnB 4.0.9; o leitor Web já
retorna nomes observados. Uma criação concorrente pode coincidir em
tipo/especificação/nível, mas diferir nos interessados.

**RED em [CI 42251678538](https://github.com/franklinbaldo/ies/actions/runs/42251678538)**:
**1 FAIL novo, 1.730 PASS, 22 ignored preexistentes**, ao classificar
como `Aplicada` um candidato com interessado divergente. Implementada
conferência por multiconjunto quando o pedido contém interessado
explícito, tanto na reconciliação quanto depois de `Location`.
Nenhum GET/POST adicional; orçamentos anteriores permanecem.

**LIVE desta fatia: NOT_RUN** — prévia 0 GET, confirmação 0 POST,
verificação 0 GET, **0 requisições reais e 0 resíduos novos**. A
criação histórica da UnB NUP `23106.001875/2026-93`, id 51731, não
foi repetida. O ensaio em segunda instalação de treinamento continua
pendente, na ordem prévia → uma confirmação → leitura e registro do resíduo.

## 🦉 Guarda de identidade anterior ao POST (06/11/2026)

A fotografia completa da caixa Gerados do Controle anterior ao POST pode
revelar que o ID de `302 Location` **já existia**. Esse retorno não atesta
nova criação, mesmo que tela, árvore e propriedades do processo antigo
coincidam com o pedido. A regressão sintética foi publicada antes da
implementação: [CI RED 42271272286](https://github.com/franklinbaldo/ies/actions/runs/42271272286),
**1 FAIL / 1.733 PASS / 22 ignored preexistentes**; a CLI antiga
respondia `confirmado: true` e NUP para identidade preexistente.
Após a guarda, o caso fica inconclusivo (saída 3), subcódigo tipado,
**3 GET iniciais + 1 POST = 4 pedidos**, sem GET adicional nem retry.

**LIVE desta fatia:** prévia **NOT_RUN (0 req.)**, confirmação única
**NOT_RUN (0 POST)**, verificação **NOT_RUN (0 req.)**;
**0 requisições reais, 0 escritas, 0 resíduos novos**. Não repetir o
NUP histórico UnB `23106.001875/2026-93` / ID 51731. A execução em
segunda instalação exclusivamente de treinamento continua pendente.
