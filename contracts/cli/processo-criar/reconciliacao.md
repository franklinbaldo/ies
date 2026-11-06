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

A confirmação atual ainda pode devolver `efeito_incerto_na_criacao`/saída 3
quando o POST criador falha ou não apresenta resultado verificável.
A reconciliação **ainda não está ligada à CLI** e não há autorização
para repetir o POST de criação da UnB que já produziu
`23106.001875/2026-93` (id 51731).

A primeira peça implementada, `src/processo_criar/reconciliacao.rs`,
é pura, sem requisições. Ela lê dois retratos da lista **Gerados** pela
infraestrutura de `paginas`, exige unidade conferida, total publicado,
intervalo integral de 1 até o total e todas as linhas; recusa cobertura
incompleta, identidade anterior alterada e desaparecimento de ID.
Compara `id_procedimento` e NUP como valores opacos e devolve **candidatos**,
sem atribuir causalidade ao POST. Lista vazia de candidatos **não**
significa que a criação certamente não ocorreu: a caixa pode variar
entre requisições e ainda falta verificar tipo/especificação em `processo NUP`.
Não expõe links assinados, cookie ou `infra_hash`.

Os oito testes sintéticos da fatia usam só HTML local, com
**zero requisições e zero POST**. Ainda faltam a captura prévia de Gerados
antes da escrita, sua leitura posterior sem repetir requests,
a verificação dos NUPs, os quatro resultados de transporte injetado
(aplicada, não aplicada, candidatos ambíguos e erro de reconciliação),
as duas causas no erro final e a prova live. Não interpretar a peça
offline como reconciliação completa.

| Subcódigo | Fato |
| --- | --- |
| `gerados_sem_cobertura_completa_para_reconciliacao` | Total ou faixa insuficiente para atestar todos os Gerados |
| `identidade_ja_observada_alterada_na_reconciliacao` | Um ID pré-existente passou a apontar outro NUP |
| `identidade_anterior_ausente_na_reconciliacao` | Um ID pré-existente desapareceu da nova lista |

### Guarda de segurança antes do POST (#245 — continuação)

Na confirmação, depois de validar o cadastro e **antes do primeiro POST
criador**, o fluxo monta uma fotografia de **Gerados** com a leitura do
Controle já efetuada. Exige mesma unidade, total declarado e cobertura
integral; legenda paginada/indeterminada recusa com causa tipada
`gerados_sem_cobertura_completa_para_reconciliacao`, saída 1, **sem
POST criador** e **sem GET extra**. A prévia permanece disponível.
Esse é um pré-requisito da reconciliação, **não** sua implementação:
ainda não há comparação posterior nem classificação automática da falha
do POST. Não interpretar a proteção como escrita homologada em live.

### Releitura conservadora pós-POST e candidatos (issue #245)

Depois de uma falha observada **após iniciar o POST criador**, o fluxo faz
**uma única nova leitura do Controle** na mesma sessão e compara Gerados
com o retrato integral anterior. Orçamento de teste: **3 GET de prévia,
1 POST criador e 1 GET de reconciliação = 5 pedidos, sem retry**.
Se até essa leitura falha, a saída é 3 e preserva as duas causas.
Se dá certo, ainda é saída 3: a diferença são **candidatos observados**,
não prova da aplicação ou de que repetir a escrita é seguro.
O JSON publica `causa_post`, `causa_releitura` e
`candidatos_observados_na_caixa` (null se releitura falhou), mas nunca
cookie, assinatura, URL ou hash.

O classificador offline `processo_criar/desfechos.rs` usa candidatos e
provas independentes de processo NUP já lidas. Ele exige ID, NUP, tipo
e especificação exatos; falta de leitura, identidade divergente,
prova duplicada e propriedades não observadas são recusadas.
Uma correspondência: `Aplicada`; duas ou mais: `CandidatosAmbiguos`;
zero: `SemCorrespondenciaObservada` — **não** prova de não aplicação.
O classificador já confere candidatos pela Web depois de resposta inconclusiva.
Ainda faltam desfechos públicos de não aplicação comprovada e ambiguidade,
além do ensaio live.

| Subcódigo | Fato |
| --- | --- |
| `candidato_sem_leitura_verificadora` | Novo candidato sem leitura de processo NUP |
| `leitura_verificadora_repetida` | Mesma identidade conferida duas vezes |
| `leitura_verificadora_de_outro_processo` | Par ID/NUP da leitura diferente do candidato |
| `propriedades_do_candidato_nao_observadas` | Tipo ou especificação não publicados na leitura |

## Resposta inesperada após POST criador (#245, fatia 2026-11-06)

Se o POST criador devolver HTML em vez de 302, ou `Location` ausente,
repetida ou inválida, o IES **não segue** esse endereço nem repete o POST.
Reconsulta **uma única vez** o Controle conhecido da sessão e compara a
fotografia completa de Gerados obtida antes do envio. Reporta a causa
nomeada `resposta_inesperada_apos_post_criador` ou
`localizacao_invalida_apos_post_criador`, os candidatos apenas observados
e a causa independente da releitura, quando esta falhar. A resposta continua
`efeito_incerto_na_criacao`/saída 3 porque diferença de Gerados, inclusive
vazia, não prova por si só criação nem não aplicação. Conferência de cada
candidato por `processo NUP` e classificação final são trabalho pendente.

### Verificação independente de novos candidatos (#245, fatia seguinte)

Depois da tentativa de POST criador, a nova leitura de Gerados identifica
os candidatos. Para **cada** ID/NUP novo, a CLI segue somente o link
de trabalho servido na linha correspondente, confere árvore e unidade e
lê propriedades (tipo e especificação). São **1 GET da caixa + 3 GET
por candidato**, sem retry. Uma única correspondência exata permite
retornar criação aplicada (saída 0), NUP e `causa_post`; se a
reconciliação falhar ou continuar ambígua, saída 3, sem autorizar
outra escrita. A prova não inclui ainda um desfecho automático
conclusivo de não aplicação. Live: NOT_RUN nesta fatia.

| Subcódigo | Fato |
| --- | --- |
| `link_do_candidato_nao_comprovado` | Link da linha Gerados ausente, fora da origem ou com identidade divergente |
| `links_do_candidato_repetidos` | Dois links para o mesmo par de Gerados |
| `propriedades_nao_oferecidas_ao_candidato` | A árvore conferida não publicou endereço de propriedades |

## Recusa explícita e concorrência (#245)

Na validação expressa da Infra (HTML 200 com recusa publicada pelo SEI),
a saída é 1, com causa original e `desfecho: "nao_aplicada"` no JSON:
`repeticao_segura_com_nova_previa: true` só permite uma nova decisão
explícita após prévia; `retentavel: false` veda retry automático.
HTTP 4xx isolado (inclusive 403) **não prova ausência de efeito**,
retorna **saída 3** (`efeito_incerto`), com subcódigo, status HTTP e
mensagem saneada, sem publicar `nao_aplicada` nem habilitar repetição.
Somente a recusa expressa da Infra retorna **saída 1**. Ambos encerram
após **3 GET + 1 POST**, sem reler Gerados nem atribuir criação
concorrente à operação recusada.
Falha de rede sem resposta e 5xx seguem reconciliação sem retry.


## Desfecho verificável de ambiguidade após POST (#245)

Se a leitura independente de **todos** os novos processos encontrar **dois
ou mais** com ID/NUP, tipo e especificação comprovados e coincidentes,
a CLI responde saída **3** com subcódigo
`candidatos_ambiguos_na_criacao`. No JSON, publica
`desfecho: "candidatos_ambiguos"` e `candidatos_compativeis` com os
NUPs observados, além de `causa_post` e da fotografia posterior.
No humano, identifica a ambiguidade e os NUPs, sem inferir o autor da
escrita nem recomendar repetição. Com dois candidatos, a prova usa
**3 GET antes + 1 POST criador + 1 GET Controle + 6 GET de processos = 11
requisições** sintéticas, sem retry. Uma única correspondência, após
verificar todos, continua permitindo saída 0; zero correspondências
**não** prova não aplicação e mantém saída 3. O desfecho de não
aplicação com prova própria continua pendente.


## Verificação após `Location` válido (#245)

Um redirecionamento esperado do POST criador não basta para afirmar
aplicação: a tela do processo e sua árvore ainda devem ser lidas.
Se um GET verificador falhar por transporte ou HTTP, a CLI conserva
**saída 3** (efeito incerto), o subcódigo original do GET, eventual
`status_http` e mensagem saneada do servidor. Uma falha estrutural
da tela ou da árvore também mantém sua causa específica; nunca se
atribui sucesso com base somente no `Location`. Não há retry nem
GET adicional de reconciliação nesta fatia.

Cenários sintéticos do primeiro GET verificador: **3 GET iniciais +
1 POST criador + 1 GET verificador falho = 5 pedidos**, exatamente
um POST. HTTP 503 permanece 503; falha de rede mantém
`falha_de_rede`. Live em outra instalação: NOT_RUN.

## Falha HTTP na leitura de reconciliação (#245)

Se o único POST criador terminar inconclusivo, uma falha HTTP do **GET
posterior do Controle** continua sendo uma causa de leitura, distinta da
causa da escrita. A saída é 3, sem retry. O JSON conserva
`causa_post`, `causa_releitura`, e acrescenta
`status_http_releitura` e `mensagem_do_servidor_releitura` **somente
quando o GET publicar esses fatos**. A mensagem já foi saneada pelo
transporte; nenhuma URL ou credencial é exposta.

O campo genérico `status_http` prioriza o status da resposta do POST,
se ele existe, e usa o status da releitura quando a tentativa de POST
não teve resposta HTTP. Assim, se o POST retornou 503 e o GET posterior
502, o diagnóstico preserva **ambos**, sem confundir os estágios.
Teste sintético: **3 GET prévios + 1 POST + 1 GET posterior = 5 pedidos**,
um POST exato, zero retry. Live em segunda instalação: NOT_RUN.


## Coincidências parciais antes da falha de outro candidato (#245)

Quando Gerados apresenta vários candidatos, cada leitura independente
concluída pode comprovar correspondência de **ID/NUP, tipo e especificação**.
Se a verificação de um candidato posterior falhar, essas coincidências
já obtidas não desaparecem: o diagnóstico publica
`coincidencias_parciais_conferidas` com os NUPs efetivamente verificados,
e mantém `candidatos_observados_na_caixa` com todos os NUPs da fotografia.
Isso **não** permite concluir que a escrita foi aplicada, nem que a
correspondência é única: saída **3**, `efeito_incerto_na_criacao`,
sem `desfecho` conclusivo e sem retry.

`candidatos_compativeis` e `desfecho: candidatos_ambiguos` permanecem
reservados ao caso em que **todos** os candidatos foram verificados e
ao menos dois coincidem. O cenário adversarial sintético usa 3 GET de
preparação + 1 POST incerto + 1 GET Controle + 3 GET do primeiro
candidato + 1 GET falho do segundo = **9 requisições**, um único POST.
Nenhum resultado parcial autoriza nova escrita.
