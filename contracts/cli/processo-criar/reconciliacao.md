---
type: Guide
title: Reconciliação da criação de processos após o POST
description: Critérios e orçamento de requisições para classificar respostas incertas da operação processo.criar
operation: processo.criar
---

# Reconciliação da criação de processos

Este complemento preserva as regras e a evolução incremental do
[contrato do comando](../processo-criar.md).

## Reconciliação automática após resposta incerta (#245 — fatia em andamento)

A confirmação atual ainda pode devolver `ErroCriar::NaoConfirmado`/saída 1
quando o POST criador falha ou não apresenta resultado verificável.
A reconciliação está ligada à CLI, com classificação conservadora dos candidatos; não há autorização
para repetir o POST de criação da UnB que já produziu
`23106.001875/2026-93` (id 51731).

A primeira peça implementada, `src/processo_criar/reconciliacao.rs`,
é pura, sem requisições. Ela lê dois retratos da lista **Gerados** pela
infraestrutura de `paginas`, exige unidade conferida, total publicado,
intervalo integral de 1 até o total e todas as linhas; recusa cobertura
incompleta, identidade anterior alterada e desaparecimento de ID.
Recusa também NUP repetido sob IDs distintos, inclusive quando a legenda
confirma todas as linhas: duplicidade de número invalida a identidade do
retrato, não apenas sua paginação. A confirmação recusa **antes do POST
criador**, com os três GETs de preparação já observados, zero escritas.
Compara `id_procedimento` e NUP como valores opacos e devolve **candidatos**,
sem atribuir causalidade ao POST. Lista vazia de candidatos **não**
significa que a criação certamente não ocorreu: a caixa pode variar
entre requisições e ainda falta verificar tipo/especificação em `processo NUP`.
Não expõe links assinados, cookie ou `infra_hash`.

Os oito testes sintéticos da fatia usam só HTML local, com
**zero requisições e zero POST**. O fluxo Web já usa captura prévia, releitura posterior e verificação dos
NUPs. As causas do POST e do GET posterior são preservadas; a presença
de candidatos não prova autoria. A validação live em uma segunda
instalação de treinamento continua pendente.

| Erro (stderr) | Fato |
| --- | --- |
| `ErroFotografia::CoberturaIncompleta` | Total ou faixa insuficiente para atestar todos os Gerados |
| `ErroFotografia::NupRepetido` | Dois IDs diferentes da mesma fotografia apontam para o mesmo NUP: a cobertura posicional não comprova identidades independentes |
| `ErroFotografia::IdentidadeAlterada` | Um ID pré-existente passou a apontar outro NUP |
| `ErroFotografia::IdentidadeAnteriorAusente` | Um ID pré-existente desapareceu da nova lista |

### Guarda de segurança antes do POST (#245 — continuação)

Na confirmação, depois de validar o cadastro e **antes do primeiro POST
criador**, o fluxo monta uma fotografia de **Gerados** com a leitura do
Controle já efetuada. Exige mesma unidade, total declarado e cobertura
integral; legenda paginada/indeterminada recusa com causa tipada
`ErroFotografia::CoberturaIncompleta`, saída 1, **sem
POST criador** e **sem GET extra**. A prévia permanece disponível.
A comparação posterior e a classificação conservadora já integram o
fluxo Web. Isso não é homologação de escrita ao vivo.

### Releitura conservadora pós-POST e candidatos (issue #245)

Depois de uma falha observada **após iniciar o POST criador**, o fluxo faz
**uma única nova leitura do Controle** na mesma sessão e compara Gerados
com o retrato integral anterior. Orçamento de teste: **3 GET de prévia,
1 POST criador e 1 GET de reconciliação = 5 pedidos, sem retry**.
Se até essa leitura falha, a saída é 1 e preserva as duas causas.
Se dá certo, ainda é saída 1: a diferença são **candidatos observados**,
não prova da aplicação ou de que repetir a escrita é seguro.
`ErroCriar::PosEscrita` carrega `causa_post` e `posterior` (os
candidatos lidos, ou o erro da releitura), mas nunca cookie, assinatura,
URL ou hash.

O classificador offline `processo_criar/desfechos.rs` usa candidatos e
provas independentes de processo NUP já lidas. Ele exige ID, NUP, tipo
e especificação exatos; falta de leitura, identidade divergente,
prova duplicada e propriedades não observadas são recusadas.
Uma correspondência: `CoincidenciaUnica` (possível resíduo, **não** autoria);
duas ou mais: `CandidatosAmbiguos`; zero: `SemCorrespondenciaObservada`
(**não** prova de não aplicação). A recusa expressa da Infra é o único caso
observado de não aplicação comprovada; falta a validação live.

| Erro (stderr) | Fato |
| --- | --- |
| `ErroConferencia::LeituraAusente` | Novo candidato sem leitura de processo NUP |
| `ErroConferencia::LeituraRepetida` | Mesma identidade conferida duas vezes |
| `ErroConferencia::IdentidadeDivergente` | Par ID/NUP da leitura diferente do candidato |
| `ErroConferencia::PropriedadesNaoObservadas` | Tipo ou especificação não publicados na leitura |

## Resposta inesperada após POST criador (#245, fatia 2026-11-06)

Se o POST criador devolver HTML em vez de 302, ou `Location` ausente,
repetida ou inválida, o IES **não segue** esse endereço nem repete o POST.
Reconsulta **uma única vez** o Controle conhecido da sessão e compara a
fotografia completa de Gerados obtida antes do envio. Reporta a causa
nomeada `resposta_inesperada_apos_post_criador` ou
`localizacao_invalida_apos_post_criador`, os candidatos apenas observados
e a causa independente da releitura, quando esta falhar. A resposta continua
`ErroCriar::NaoConfirmado`/saída 1 porque diferença de Gerados, inclusive
vazia, não prova por si só criação nem não aplicação. A conferência de cada candidato por `processo NUP` e a classificação
conservadora já estão ligadas ao fluxo Web.

### Verificação independente de novos candidatos (#245, fatia seguinte)

Depois da tentativa de POST criador, a nova leitura de Gerados identifica
os candidatos. Para **cada** ID/NUP novo, a CLI segue somente o link
de trabalho servido na linha correspondente, confere árvore e unidade e
lê propriedades (tipo e especificação). São **1 GET da caixa + 3 GET
por candidato**, sem retry. A conferência prova a identidade e
as propriedades de um **candidato**, mas não atribui causalidade ao POST
sem resposta em uma conta compartilhada: um agente concorrente poderia
ter criado um processo indistinguível na mesma janela. Mesmo havendo
exatamente um correspondente, o resultado é saída 1, possível resíduo
e causa do POST preservada. Duas ou mais coincidências são ambíguas;
nenhuma ou leitura interrompida continua inconclusiva. Somente um
`Location` legítimo vinculado ao POST e verificado por leituras
independentes pode produzir sucesso. A recusa literal da Infra é
a prova distinta de não aplicação. Live: NOT_RUN nesta fatia.

| Erro (stderr) | Fato |
| --- | --- |
| `LinkDoCandidatoNaoComprovado` | Link da linha Gerados ausente, fora da origem ou com identidade divergente |
| `LinksDoCandidatoRepetidos` | Dois links para o mesmo par de Gerados |
| `ErroCriar::PropriedadesNaoOferecidas` | A árvore conferida não publicou endereço de propriedades |

## Recusa explícita e concorrência (#245)

Na validação expressa da Infra (HTML 200 com recusa publicada pelo SEI),
a saída é 1 com `FalhaNaCriacao { causa: RecusaDoPost(ValidacaoRecusadaPeloSei(..)), requisicoes }`:
só uma nova decisão explícita após prévia pode repetir; nada é repetido
automaticamente.
HTTP 4xx isolado (inclusive 403) **não prova ausência de efeito**:
**saída 1** com `RespostaHttpSemProva(Transporte(Status(StatusHttpRecusado { status, mensagem_do_servidor })))`,
sem habilitar repetição.
Somente a recusa expressa da Infra retorna **saída 1**. Ambos encerram
após **3 GET + 1 POST**, sem reler Gerados nem atribuir criação
concorrente à operação recusada.
Falha de rede sem resposta e 5xx seguem reconciliação sem retry.


## Desfecho verificável de ambiguidade após POST (#245)

Se a leitura independente de **todos** os novos processos encontrar **dois
ou mais** com ID/NUP, tipo e especificação comprovados e coincidentes,
a CLI responde saída **1** com
`ErroCriar::PosEscrita { causa_post, posterior, correspondentes: Some(nups), .. }`
(dentro de `FalhaNaCriacao`): os NUPs compatíveis, a causa do POST e a
fotografia posterior ficam nos campos, sem inferir o autor da escrita
nem recomendar repetição. Não há mais rótulo derivado (`desfecho`) na
saída: um NUP em `correspondentes` é coincidência sem prova causal, dois
ou mais são ambiguidade. Com dois candidatos, a prova usa
**3 GET antes + 1 POST criador + 1 GET Controle + 6 GET de processos = 11
requisições** sintéticas, sem retry. Uma única correspondência após verificar todos **continua saída 1**
(`possivel_residuo`) sem resposta causal do POST; zero correspondências
**não** prova não aplicação e mantém saída 1. Recusa expressa da Infra
é prova distinta de não aplicação.


## Verificação após `Location` válido (#245)

Um redirecionamento esperado do POST criador não basta para afirmar
aplicação: a tela do processo e sua árvore ainda devem ser lidas.
Se um GET verificador falhar por transporte ou HTTP, a CLI conserva
**saída 1** (escrita não confirmada), o subcódigo original do GET, eventual
`status_http` e mensagem saneada do servidor. Uma falha estrutural
da tela ou da árvore também mantém sua causa específica; nunca se
atribui sucesso com base somente no `Location`. Não há retry nem
GET adicional de reconciliação nesta fatia.

Cenários sintéticos do primeiro GET verificador: **3 GET iniciais +
1 POST criador + 1 GET verificador falho = 5 pedidos**, exatamente
um POST. HTTP 503 permanece 503; falha de rede mantém
`Transporte(Rede)`. Live em outra instalação: NOT_RUN.

## Falha HTTP na leitura de reconciliação (#245)

Se o único POST criador terminar inconclusivo, uma falha HTTP do **GET
posterior do Controle** continua sendo uma causa de leitura, distinta da
causa da escrita. A saída é 1, sem retry. `ErroCriar::PosEscrita`
conserva `causa_post` e, em `posterior`, o erro da releitura com o seu
`Status(StatusHttpRecusado { status, mensagem_do_servidor })` **somente
quando o GET publicar esses fatos**. A mensagem já foi saneada pelo
transporte; nenhuma URL ou credencial é exposta.

Os dois status ficam em lugares distintos: o do POST em `causa_post`, o
da releitura em `posterior`. Assim, se o POST retornou 503 e o GET posterior
502, o diagnóstico preserva **ambos**, sem confundir os estágios.
Teste sintético: **3 GET prévios + 1 POST + 1 GET posterior = 5 pedidos**,
um POST exato, zero retry. Live em segunda instalação: NOT_RUN.


## Coincidências parciais antes da falha de outro candidato (#245)

Quando Gerados apresenta vários candidatos, cada leitura independente
concluída pode comprovar correspondência de **ID/NUP, tipo e especificação**.
Se a verificação de um candidato posterior falhar, essas coincidências
já obtidas não desaparecem: `ErroCriar::PosEscrita` traz em
`correspondentes` os NUPs efetivamente verificados e em
`observados_antes_da_falha` todos os candidatos da fotografia.
Isso **não** permite concluir que a escrita foi aplicada, nem que a
correspondência é única: saída **1**, `ErroCriar::PosEscrita`, sem
conclusão e sem retry.

Ambiguidade (dois ou mais em `correspondentes`, sem
`observados_antes_da_falha`) fica reservada ao caso em que **todos** os
candidatos foram verificados e ao menos dois coincidem. O cenário adversarial sintético usa 3 GET de
preparação + 1 POST incerto + 1 GET Controle + 3 GET do primeiro
candidato + 1 GET falho do segundo = **9 requisições**, um único POST.
Nenhum resultado parcial autoriza nova escrita.

## Nível de acesso como parte da prova de aplicação (#245)

A leitura independente do candidato confirma também o **nível de acesso**
observado em `rdoNivelAcesso`, além de ID/NUP, tipo e especificação.
Um processo concorrente com mesmo tipo e especificação, mas nível
`restrito` quando o pedido foi `publico` (ou vice-versa), **não**
comprova esta escrita. Nível ausente tampouco autoriza conclusão positiva
ou negativa. O classificador retorna `SemCorrespondenciaObservada` para
nível divergente, e a CLI mantém saída 1 com a fotografia posterior.

O orçamento não muda: **3 GET de preparação + 1 POST tentado + 1 GET
do Controle + 3 GET de verificação = 8 pedidos**, exatamente um POST,
sem retry. Essa regra também vale para as coincidências parciais em
caso de falha ao verificar outro candidato.

## Interessados explícitos na prova de criação (#245)

Se o pedido incluiu `--interessado`, uma correspondência somente de
tipo, especificação e nível **não** comprova o mesmo pedido: outro agente
pode criar um processo semelhante para outra pessoa. A leitura Web de
propriedades já publica `selInteressadosProcedimento`. A confirmação
pós-`Location` e a reconciliação só admitem correspondência quando a
lista observada contém **exatamente os interessados solicitados**, sem
depender da ordem, mas preservando multiplicidade. Nenhum interessado
explícito significa que o pedido não impôs essa condição adicional.

Divergência não prova não aplicação: no caminho pós-`Location`, saída
3 com a causa de divergência; na reconciliação, candidato não coincidente
e saída 1 se faltar outra prova. A conferência reutiliza o terceiro GET
de propriedades, **sem novo request, POST, retry ou execução de JS**.

## `Location` aponta para ID anterior ao POST (#245)

Mesmo com `302 Location` sintaticamente válido, tela, árvore e propriedades
coincidentes, não se declara **criação nova** se o `id_procedimento`
retornado já consta da fotografia **completa** dos Gerados anterior ao POST.
Esta evidência negativa evita atribuir uma identidade antiga ao POST
presente. O resultado é escrita não confirmada (**saída 1**), subcódigo
`id_preexistente_no_retorno_criador`, sem recomendar repetição,
sem abrir o processo preexistente e sem tentar outro POST. O caso adversarial
sintético exige **3 GET iniciais + 1 POST = 4 requisições**,
zero GET de verificação supérfluo, zero retry. A fotografia não é
prova universal de causalidade, mas é uma condição necessária adicional.

## Concorrência indistinguível: candidato único não prova autoria (06/11/2026)

Depois de um POST sem resposta, mesmo a diferença entre fotografias
completas de Gerados seguida de três GET por candidato, confirmação de
ID/NUP, tipo, especificação, nível e interessados só demonstra
**coincidência**. Sem o recibo causal da escrita, outra sessão
pode produzir exatamente a mesma diferença durante a janela observada.

O classificador tipado devolve `CoincidenciaUnica` (fato sobre
propriedades), mas a fronteira de escrita **não** define
`confirmado: true`: devolve código 1 com `ErroCriar::PosEscrita`
(`correspondentes: Some([nup])`, a causa original do POST) e proibição
de retry.
Com dois candidatos dos quais só um coincide, lê **ambos** antes de
classificar; não atribui o correspondente à operação. As coincidências
parciais após falha de outra leitura permanecem sem desfecho conclusivo.

Testes sintéticos verificam 8 pedidos com um candidato (3 GET iniciais,
1 POST, 1 GET Controle posterior, 3 GET propriedades/árvore), e 11
pedidos com dois candidatos, **sempre 1 POST criador, sem retry**.
A mudança não envia requisições novas. A validação real em segunda
instalação de treinamento continua NOT_RUN, e o POST histórico da UnB
não deve ser repetido.

[Decisão de concorrência causal](../../../docs/decisoes/processo-criar/concorrencia-pos-post.md)
· [Evidência RED e live NOT_RUN](../../../docs/validacao-live/2026-11-06-processo-criar-concorrencia-causal.md)

## Contagem efetiva de requisições em erros (#245)

O campo JSON `requisicoes` e a mensagem humana refletem o número
**efetivamente tentado** de chamadas ao transporte na operação: incluem
GETs de preparação, o único POST criador, eventual AJAX de consulta,
releitura de Gerados e verificações independentes já realizadas. Não
são orçamento estimado nem prova de aplicação. Mesmo com saída 1
por escrita não confirmada, a contagem é conservada; nunca habilita retry.
Falhas antes da primeira chamada publicam zero.

Regressões offline: prévia 3 GET/0 POST; recusa de unidade após 1 GET;
falha de POST com candidato coincidente e não causal, 8 chamadas e
**exatamente 1 POST**, sem retry. RED no CI 42305129633:
3 testes novos falharam, 1.773 passaram e 22 preexistentes ignorados.
LIVE em segunda instalação de treinamento: NOT_RUN.
