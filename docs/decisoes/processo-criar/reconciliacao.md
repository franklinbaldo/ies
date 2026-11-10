---
type: ImplementationDecision
title: Reconciliação e resposta do criador na interface Web
operation: processo.criar
implementation: partial
status: adopted-for-scope
scope: retratos-gerados-verificacao-web-e-recusa-explicita
selected_strategy: leitura-do-controle-e-conferencia-dos-candidatos
alternatives: repetir-post,atribuir-candidato-sem-prova,conferir-retratos-e-identidades
evidence_level: testes-sinteticos-e-replay-publico
live_validation: not-run
---

# Decisão de reconciliação da criação de processo — #245

Continuação da [decisão de criação](../processo-criar.md), sem alterar os
critérios de prévia, seleção do tipo ou submissão única.

## Continuação da reconciliação: retratos de Gerados (#245)

A conciliação histórica **já confirmou por leitura** que a única
confirmação incerta da UnB de treinamento foi aplicada:
NUP `23106.001875/2026-93`, id `51731`
([evidência](../../validacao-live/2026-11-05-processo-criar-reconciliacao.md)).
Essa confirmação **não será repetida**. O relato anterior de efeito
inconclusivo se refere à resposta da CLI no momento do POST, não ao
resultado conhecido posteriormente.

Para o mecanismo automático ainda pendente, a primeira fatia escolhe
reutilizar `paginas::juntar` e a tabela tipada de Gerados em vez de
fazer novo parser ou inferir IDs a partir de NUPs.
A fotografia exige cobertura integral de uma página, unidade
coincidente e IDs anteriores estáveis. Uma página paginada é
**inconclusiva**, mesmo com ausência aparente de candidatos.
Um ID que desapareceu ou mudou de NUP também impede conclusão.
Quando há dois candidatos, ambos permanecem candidatos, sem escolher
um por ordem nem inferir autoria da criação. A diferença é uma
**observação**, não ainda prova de aplicação ou de segurança de retry.

Foram considerados (1) comparar apenas a primeira página sem verificar
total, (2) inferir criação pelo maior ID e (3) comparar tabelas tipadas
com cobertura demonstrada. Escolhida a terceira alternativa: mais
restritiva, porém evita falso negativo após uma escrita.
A próxima etapa deve obter Gerados antes/depois, verificar tipo e
especificação dos candidatos por leitura independente e preservar a
causa original do POST sem expor URLs/cookies. Os quatro desfechos
exigidos pela issue continuam pendentes; não liberar novos
`--confirmar` para ensaios de escrita não confirmada.

### Guarda anterior à escrita, continuação da #245

A observação indireta do Controle mostrou que a tabela Gerados
publica total/faixa, mas pode apresentar só uma página. Antes de enviar
uma escrita cuja resposta talvez precise ser reconciliada, exigir
fotografia íntegra da leitura inicial, reutilizando
`FotografiaDosGerados::da_tela` (0 requisições adicionais).
Alternativas: escrever mesmo com lista parcial (não garante candidatos
posteriores), percorrer automaticamente páginas (mais requests e
muda a tela da sessão), ou recusar de forma tipada. Foi escolhida a
recusa segura; **`--confirmar` não envia POST** quando a cobertura
é insuficiente, mas prévias só de leitura continuam disponíveis.
Os testes novos verificam 3 GET / 0 POST na recusa e na prévia.
A fotografia é preservada no escopo do fluxo, ainda não consumida
após falha: falta a reconciliação Web completa e a verificação independente
de tipo/especificação, sem repetir escritas. Esta guarda restringe
a confirmação e não constitui validação autenticada ao vivo.

### Releitura única depois de falha do POST — 06/11 (#245)

A evidência do ensaio UnB anterior mostrou uma criação real mesmo
quando o POST voltou à CLI com saída 1. Foi rejeitada a alternativa
de repetir o POST, seguir o redirect ou afirmar não aplicação só porque
o NUP não surgiu na resposta. O caminho escolhido reutiliza a
fotografia completa prévia de Gerados e, exclusivamente após falha
de submissão já iniciada, lê uma única vez o Controle para obter
candidatos novos, preservando a causa original da falha e também
a da releitura, quando houver. Contagem sintética: 5 requests
(3 GET, 1 POST criador e 1 GET de conferência).

Esse passo é conservador: mesmo com zero candidatos, não se afirma
que a escrita falhou; outros processos e efeitos de sessão podem
interferir. A saída permanece 3 até leitura verificadora de cada
candidato com ID/NUP, tipo e especificação. O classificador puro
exige essas quatro provas, rejeita parcialidade e mantém ambiguidade
quando vários candidatos correspondem. Ainda falta ligar
o classificador ao transporte e à apresentação dos quatro desfechos.
Nenhuma escrita, leitura ou ensaio live novo foi efetuado nesta fatia.

## Resposta do criador sem identidade (fatia de 06/11/2026)

Adotada uma reconciliação conservadora também para respostas não excepcionais
**inesperadas** após o POST criador: HTML 200 em lugar de redirecionamento, ou
302 com `Location` inválida/ambígua. Sem seguir a `Location`, o fluxo pede
exatamente uma leitura do Controle originalmente observado, compara as
fotografias completas dos Gerados e preserva a causa distinta da resposta e
a falha da releitura, quando houver. Um único POST; nenhum retry. Publicar
candidatos não é inferir efeito. Por isso a saída permanece 3 até conferir
propriedades de todos os candidatos e decidir o resultado; a reconciliação
automática conclusiva segue pendente. Essa alternativa usa a página de
Controle já conhecida, sem tentar inventar o endereço do processo novo.

### Candidatos verificados pela Web (#245)

A próxima fatia usa links **servidos na linha Gerados**, nunca links
inferidos de IDs, e os leitores já disponíveis de tela, árvore e
propriedades. Para cada candidato é necessário comprovar unidade,
ID/NUP, tipo e especificação. A correspondência exata com todos os
candidatos conferidos é necessária, mas **não prova a autoria de um POST
sem resposta** numa conta compartilhada. Mesmo se for única, o resultado
permanece inconclusivo (saída 1), como detalha a
[decisão causal](concorrencia-pos-post.md). Sem propriedades publicadas,
falha fechado e preserva a causa do POST.
O novo leitor consome 1 GET da caixa e 3 GET por candidato; não faz
retry nem novo POST. A ausência de candidatos não prova não aplicação.
O ensaio live nesta fatia não foi executado.

### Recusa explícita e concorrência (06/11, #245)

Decisão: validação Infra ou HTTP 4xx após POST encerra com a causa
original, sem GET de Gerados; um candidato idêntico pode ser de outro
agente. Sem resposta e 5xx seguem reconciliação sem retry. Teste RED
adversarial com a conta compartilhada precedeu a correção.


### Classificação de múltiplos candidatos (06/11, #245)

Quando a releitura observa dois processos novos e a conferência da
identidade, da unidade, do tipo e da especificação confirma **ambos**,
nenhum deles prova autoria do POST que falhou. Adotado um desfecho
público distinto, `candidatos_ambiguos_na_criacao`, com saída 1,
NUPs comprovadamente compatíveis e causa do POST preservada.
Alternativas rejeitadas: escolher o primeiro e contar candidatos como
prova de criação; ou esconder a ambiguidade sob o diagnóstico genérico.
Caso haja uma só correspondência, todas as demais candidatas ainda
precisam de leitura independente antes da conclusão. Se não houver
correspondência, não afirmar não aplicação nem permitir retry.

A fatia é suportada por dois cenários de transporte sintético de 11
requisições, cada qual com um único POST. O ensaio ao vivo em segunda
instalação de treinamento permanece NOT_RUN: zero requisições e escritas.

### Não aplicada: recusa comprovada pelo próprio SEI (#245)

A tela de validação da Infra em HTTP 200 publica uma rejeição expressa
em `txaInfraValidacao`; esse fato é diferente de falha de rede ou de
status 403 produzido possivelmente por um intermediário. Apenas a
primeira prova permite emitir `desfecho: nao_aplicada` com saída 1,
causa original e a orientação de **nova prévia antes de nova confirmação**.
`retentavel: false` permanece: não há retry automático. Status 4xx sem
essa prova conserva o subcódigo/status e retorna **saída 1 (efeito
incerto)**, sem desfecho de não aplicação nem autorização de retry;
não se consulta Gerados após esses HTTP 4xx porque candidato concorrente
não deve virar sucesso da nossa operação.
Alternativa rejeitada: concluir não aplicação a partir de uma fotografia
sem novos candidatos, ou de qualquer HTTP 4xx.

Teste sintético de transporte: 3 GET antes + 1 POST (exatamente quatro
pedidos), zero GET posterior, tanto com recusa Infra quanto HTTP 403.
O RED que precedeu esta distinção registrou 2 FAIL, 1.679 PASS e
18 testes ignorados preexistentes no
[CI 42187792547](https://github.com/franklinbaldo/ies/actions/runs/42187792547).
A escrita live em outra instalação de treinamento ainda está NOT_RUN.

### Observações preservadas mesmo se a conferência falhar (06/11, #245)

A fotografia posterior completa prova quais NUPs foram observados na
caixa Gerados, não sua autoria. Se a conferência de um candidato falhar
por transporte, HTML ou propriedades, a saída 1 preserva a causa da
falha **e** os candidatos já identificados. Antes, esse erro descartava
o retrato e publicava `null` como se o Controle não tivesse sido lido.
A alternativa de tratar o candidato como criação foi rejeitada: não há
prova independente de tipo, especificação nem causalidade. O orçamento
de regressão é 6 requisições (3 GET + 1 POST + 1 GET Controle + 1 GET
verificador que falha), com um POST único e sem retry.


### Causa da verificação pós-redirecionamento (06/11, #245)

Um POST que devolveu `Location` válido ainda precisa da leitura
independente para provar o NUP. Antes, erro de rede ou HTTP no GET
verificador era descartado e substituído por `nao_confirmado_na_criacao`,
sem conservar status HTTP. A decisão foi manter um invólucro
`VerificacaoAposPost` para assinalar escrita não confirmada (saída 1), mas
propagar a causa tipada da leitura, status HTTP e mensagem saneada.
Também se propagam erros estruturais da tela/árvore, sem reduzir todas
as falhas a um erro genérico. A alternativa de confiar no `Location`
foi recusada, assim como repetir a criação.

Observação anterior: fluxo 302 + dois GET verificadores da evidência
Web existente; os casos de falha são injeções sintéticas, **não**
ocorrências live adicionais. RED no
[CI 42210005212](https://github.com/franklinbaldo/ies/actions/runs/42210005212):
2 FAIL novos, 1.705 PASS, 22 ignored preexistentes (status 503
perdido e `falha_de_rede` substituída pelo genérico).
Orçamento: 3 GET + 1 POST + 1 GET de verificação, sem repetição.

### Causas HTTP de dois estágios (06/11/2026, #245)

Observação prévia: o transporte já distingue `FalhaTransporte::Rede`
de `StatusHttpDeErroDoServidor` e conserva mensagem saneada. O caminho
de reconciliação guardava a causa do POST e a variante da releitura, mas
o método `status_http` consultava **apenas a primeira causa**. Quando
o POST falhava sem status e o GET Controle retornava 503, o HTTP 503
realmente observado desaparecia da saída.

Alternativas: atribuir o status do GET ao POST (falso fato), descartar
o status como antes (perda de diagnóstico), ou preservar o contexto das
duas operações. Escolhida a terceira: `status_http` dá preferência à
resposta do POST e, na ausência dela, consulta a releitura;
`status_http_releitura` expõe separadamente o segundo estágio quando
observado. A mensagem da releitura vem do saneador existente. Sem
requisição nova, sem retry e sem alterar a classificação de efeito
incerto (saída 1).

RED test-first no CI 42217861035: **1 FAIL novo, 1.708 PASS, 22 ignored
preexistentes**, falha `None != Some(503)` no teste de releitura
posterior ao POST sem resposta. Cenário adicional cobre 503 do POST e
502 do GET, com orçamento sintético de 5 pedidos, um único POST.
GREEN e ensaio live dependem de confirmação posterior.


### Não descartar provas parciais após falha de outro candidato (06/11, #245)

Observação pré-RED: cada candidato da fotografia Gerados é verificado
sequencialmente por tela de trabalho, árvore e propriedades. Uma falha
de rede na leitura do **segundo** candidato apagava a correspondência
que já fora comprovada para o primeiro. Alternativas: descartar todas
as provas (perda de fatos), declarar aplicação com a primeira prova
(falsa unicidade), ou conservar somente as coincidências verificadas
sem decidir a aplicação. Escolhida a terceira, por `publicar só o
observado`.

A distinção entre resultado completo e parcial é estrutural: uma
leitura que falhou permanece `Err`, mesmo com coincidências anteriores.
A saída é 3 com subcódigo `nao_confirmado_na_criacao`; JSON usa
`coincidencias_parciais_conferidas`, não
`desfecho: candidatos_ambiguos`. O humano esclarece que não há prova
de unicidade nem autoria. Nenhum GET/POST adicional, retry ou
relaxamento de validação. RED sintético:
[CI 42226093955](https://github.com/franklinbaldo/ies/actions/runs/42226093955),
**1 FAIL / 1.711 PASS / 22 ignored preexistentes** na asserção
que exigia o NUP comprovado. Budget: 9 pedidos, um POST.


### Prova completa após Location válido (06/11, #245)

Mesmo quando o POST recebe `302 Location` válido e a árvore apresenta
ID/NUP coerentes, esses dois fatos não comprovam que o processo contém
o tipo, a especificação e o nível de acesso pedidos. A alternativa de
considerar o NUP suficiente publicava `confirmado: true` antes da leitura
das propriedades. A escolha agora é reutilizar o leitor de propriedades
existente, a partir do link servido na árvore (sem construir URL), e
comparar esses três campos antes de confirmar. Falta do link, falha da
leitura ou divergência encerram com saída 1 e causa tipada; sem retry.

O novo teste adversarial usa 3 GET de preparo, 1 POST de criação,
1 GET de tela, 1 GET de árvore e 1 GET de propriedades (7 requisições
sintéticas). O RED do teste anterior ao código foi observado no
[CI 42234422953](https://github.com/franklinbaldo/ies/actions/runs/42234422953):
**1 FAIL / 1.718 PASS / 22 ignored preexistentes**; o teste encontrou
uma resposta adicional não consumida, evidenciando que o leitor parava
na árvore. As verificações positivas sintéticas 4.0, 4.1 e restrita
passam a ter três leituras pós-POST em vez de duas. O comportamento ao
vivo em segunda instalação continua NOT_RUN.

## Conservação do nível de acesso na reconciliação (06/11/2026)

Um candidato independente pode ser criado simultaneamente por outro
agente na mesma unidade, com o mesmo tipo e especificação, porém outro
nível de acesso. A reconciliação até então considerava somente os dois
primeiros campos e atribuía ao POST falho o NUP concorrente. A correção
reaproveita o nível já lido na terceira requisição verificadora; não
introduz GET, POST, retry nem novos parâmetros públicos. O classificador
puro só admite correspondência se **tipo, especificação e nível**
coincidirem; leitura incompleta continua inconclusiva.

Alternativa recusada: usar nível divergente como prova de não aplicação.
Não elimina a possibilidade de escrita não observada, portanto manter
`SemCorrespondenciaObservada`/saída 1 é necessário. Na ausência de
nova prova, não anunciar repetição segura.

## Identidade dos interessados na prova de escrita (06/11, #245)

A tela de propriedades observada na UnB 4.0.9 contém o select
`selInteressadosProcedimento`, que o leitor `propriedades_processo`
já interpreta sem rede adicional. Na reconciliação da criação,
tipo/especificação/nível iguais podiam coincidir com um processo
concorrente para interessado diferente. Escolha: quando há interessado
explícito no pedido, comparar o multiconjunto de nomes publicados,
desconsiderando apenas a ordem da seleção. Aplicar a mesma guarda à
prova após `Location` e às coincidências parciais de outros candidatos.

Alternativas recusadas: confiar somente no NUP ou tipo/especificação,
atribuir autoria por aproximação e pedir nova tela. Uma divergência não
prova que o POST falhou: o fluxo permanece inconclusivo, saída 1,
sem retry. Na ausência de interessados explícitos, não se inventa uma
restrição. Testes usam apenas fixtures sintéticas já derivadas de
estruturas observadas; nenhum ensaio live adicional.

### `Location` que reutiliza identidade já existente (06/11, #245)

Observação anterior ao RED: o Controle já fornece fotografia completa dos
Gerados antes da escrita, e a resposta 302 do criador contém o ID da árvore.
Uma resposta pode referenciar processo que já estava na fotografia; conferir
apenas ID/NUP, propriedades, tipo, nível e interessados não prova que o
**único POST** o criou. Alternativas: confiar no 302 e propriedades
(falso positivo), reler Gerados novamente (requisição extra sem superar
a identidade anterior), ou rejeitar o sucesso quando o ID já foi visto.
Escolhida a última: causa tipada `id_preexistente_no_retorno_criador`,
saída 1, sem abrir o processo antigo, sem retry. RED test-first no
[CI 42271272286](https://github.com/franklinbaldo/ies/actions/runs/42271272286):
**1 FAIL novo, 1.733 PASS e 22 ignorados preexistentes**; o sistema
publicava `confirmado: true` para o NUP preexistente. Budget após
correção: 3 GET prévios + 1 POST, nenhum GET posterior.

## NUP repetido sob IDs diferentes — 06/11/2026 (#245)

A lista Gerados pode declarar cobertura completa das linhas e ainda oferecer
identidade contraditória: dois IDs associados ao mesmo NUP. O parser genérico
`Tabela::nova` já recusa ID repetido, mas permite números iguais sob IDs
distintos, porque outras leituras não devem inventar exclusividade global.

Para a **fotografia de reconciliação da criação**, dois IDs com o mesmo NUP
não são candidatos independentes. Rejeitada a alternativa de sobrescrever
silenciosamente a identidade no mapa ou de tratar o mesmo NUP como duas
criações. A decisão acrescenta somente nessa fronteira a causa tipada
`nup_repetido_na_fotografia_de_gerados`. Antes do POST criador, falha
fechado após os **3 GET de preparação** do cenário sintético, sem escrita;
a prévia sem confirmação não muda. A regressão foi observada com teste
sintético RED no [CI 42286047215](https://github.com/franklinbaldo/ies/actions/runs/42286047215):
**1 falha nova, 1.735 testes passando e 22 ignorados preexistentes**.
A validação autenticada em uma segunda instalação de treinamento continua
**NOT_RUN**; não repetir a escrita histórica inconclusiva da UnB.
