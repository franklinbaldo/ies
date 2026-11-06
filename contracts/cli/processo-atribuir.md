---
type: CommandContract
title: Atribuir ou remover atribuição do processo na unidade ativa
operation: processo.atribuir
command: ies processo atribuir NUP (--usuario LOGIN | --remover) [--confirmar]
effect: write-process-state
implementation: partial
---

# Atribuir processo — recorte Web (Refs #264)

A interface `ies processo atribuir NUP --usuario LOGIN` prepara a atribuição
a um login da lista Web; `--remover` prepara a remoção. Os dois são
mutuamente exclusivos. NUP inválido sai com 2, antes de ler arquivos.

## Fatos conferidos

A unidade da sessão vem da tela autenticada. A árvore publica, por
unidade, o atributo `atribuido_a`. O link da barra
`procedimento_atribuicao_cadastrar` precisa ter assinatura e identidade
do processo. A tela tem `frmAtividadeAtribuir`, POST, um
`hdnIdProtocolo`, um `hdnInfraTipoPagina=2`, um `sbmSalvar`, um
`selAtribuicao` simples e único item `value="null"` habilitado e
selecionado. Usuários são `sigla - nome`, com valor numérico, e apenas
o login **inteiro e exato** escolhe uma opção.

Falta ou duplicação de login ou ID, tela fora da forma, ação de outro processo,
situação não observada e atribuição já aplicada são recusas tipadas.
Quando a situação não está observada na unidade da **tela autenticada**,
a recusa conserva `situacao_da_atribuicao_nao_observada` e publica
`unidade` (tela), `aberto_em` (siglas da árvore, `null` se não lidas),
`motivo` da leitura (`null` quando as unidades foram lidas) e
`requisicoes` efetivamente tentadas até a recusa. A presença de PR-P
na árvore não autoriza escolher PR-P se a tela publicou outra unidade.
Este diagnóstico é só leitura, não relaxa a validação nem habilita escrita.
Nenhum URL assinado, cookie ou senha é publicado. O subcódigo
`charset_da_tela_nao_suportado` indica MIME sem codificação suportada.

## Estados desta fatia

A **prévia sem --confirmar** é leitura Web com contagem de requisições
(até a abertura da tela de atribuição); devolve atual, solicitado e valor
da opção, sem POST. Também deriva o payload pelo formulário servido
(botão Salvar, ocultos originais e `selAtribuicao` exato) e recusa método,
charset ou codificação incompatíveis, sem escrever. ID numérico que aparece
para dois logins diferentes é recusado: não há identidade unívoca.
O fluxo interno injetável faz **um único POST** e lê novamente a árvore para
conferir a atribuição do processo, inclusive se a resposta do POST falhar.
Se a atribuição relida for diferente, faltar a unidade ou a leitura falhar,
o desfecho é `efeito_incerto_na_atribuicao` com saída 3 e sem retry.
Uma recusa HTTP 5xx na releitura **interrompe** a resolução pelo índice,
sem buscar outra rota por tratá-la como link envelhecido. A falha tipada
preserva o status HTTP e a mensagem saneada da releitura quando a resposta
do POST não os forneceu, inclusive após erro de rede no POST. O JSON de
`efeito_incerto_na_atribuicao` publica também `requisicoes` com a contagem
real tentada, incluindo o POST único e o GET independente de prova;
a contagem não implica que a escrita foi aplicada.
**Releitura coincidente não converte recusa do POST em sucesso.** HTTP 403,
validação expressa da Infra (ainda que HTTP 200), demais respostas recusadas
ou um 302 com destino inesperado continuam como efeito incerto (saída 3),
mesmo quando outro agente deixou a árvore no estado solicitado. A causa,
o status e a mensagem saneada da recusa são preservados. Só a resposta sem
recusa (HTML 200 ou 302 reconhecido) ou falha de rede sem resposta admite
conciliação por leitura; nenhuma dessas vias repete o POST. O 302
reconhecido aponta para `arvore_visualizar` ou `principal` no mesmo
controlador, sempre com **`id_procedimento` explícito e igual ao processo
solicitado**, sem publicar outra ação de origem. As duas ações são
genéricas sem identidade: a releitura coincidente pode ser efeito de
outro agente, inclusive na remoção. Sem ID, o resultado é efeito incerto
(saída 3), sem novo POST. A forma de `arvore_visualizar` com ID foi
observada no caso `ACEITE` do Todos @323eb01.
**Na CLI pública, --confirmar continua temporariamente bloqueado** por
`escrita_de_atribuicao_ainda_nao_verificada`, antes de abrir sessão ou
rede. Os testes de POST único e releitura já passaram; a habilitação
pública ainda depende de validação live autorizada em treinamento.

O fluxo de escrita injetável utiliza o formulário servido como fonte do
payload, exatamente um POST sem retry e uma árvore nova para verificação.
A habilitação da escrita na CLI depende ainda de ensaio seguro em treinamento,
com prévia, confirmação única e registro da atribuição ou remoção residual. A autorização se restringe às instalações
de treinamento; ordem prévia → confirmar uma vez → leitura de prova.

## Orçamento conhecido e pendências

Prévia: abertura do processo pelo índice/Controle, mais 1 GET da tela.
A escrita no fluxo de transporte acrescenta 1 POST e 1 GET de verificação,
sem repetir o POST mesmo sob falha de rede. O teste sintético a frio afirma
**5 GET + 1 POST = 6 requisições**, para atribuição e remoção; os caminhos
com índice local precisam de testes específicos.
Os testes de transporte sintético da prévia usam exatamente 4 GET e 0 POST a frio (3 da abertura e 1 da tela); barra ausente custa 3 GET e 0 POST. As recusas HTTP da prévia preservam status e mensagem saneada. Testes sintéticos do corpo do POST e da remoção usam 0 requisições:
`hdnInfraTipoPagina`, `sbmSalvar`, `selAtribuicao`, `hdnIdProtocolo`
nessa ordem. A CLI `--confirmar` continua fechada antes da sessão.
RED de transporte: [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454), **4 testes novos FAIL, 1.689 PASS**.
Replay obrigatório das sete cassettes: **PASS** no [CI 42201836542](https://github.com/franklinbaldo/ies/actions/runs/42201836542), com 42 respostas classificadas, 28 formulários GET aceitos e 14 POSTs históricos sem prova de efeito; **0 HTTP real**. Prévia live na UnB: **PASS** no [ensaio 42205702494](https://github.com/franklinbaldo/ies/actions/runs/42205702494), com login de 5 requisições e 4 GET de prévia, **0 POST de atribuição e 0 resíduo**. Segunda instalação e escritas/remoções live: **NOT_RUN**, sem requisições de atribuição.
RED da regressão de `principal` sem identidade: [CI 42204843683](https://github.com/franklinbaldo/ies/actions/runs/42204843683), **1 FAIL novo, 1.706 PASS, 23 ignored preexistentes**. Nenhuma escrita no SEI.

[Decisão](../../docs/decisoes/processo-atribuir.md),
[casos no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py),
[cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).