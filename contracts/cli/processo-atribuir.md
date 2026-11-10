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

O formulário e os controles efetivamente acionáveis devem estar disponíveis:
`hidden` no formulário, botão Salvar ou `select`, assim como um
`fieldset disabled` interno, recusam a prévia antes de qualquer POST.
A recusa é `tela_da_atribuicao_fora_da_forma`, sem novo subcódigo.

Falta ou duplicação de login ou ID, tela fora da forma, ação de outro processo,
situação não observada e atribuição já aplicada são recusas tipadas.
Quando a árvore já publica o mesmo login solicitado, ou atribuição vazia na
remoção, a recusa `AtribuicaoJaAplicada` ocorre antes do GET do formulário:
3 GET e 0 POST a frio, também na confirmação injetável. A recusa não presume
situação ausente a partir de uma unidade não observada.
Quando a situação não está observada na unidade da **tela autenticada**,
a recusa conserva `SituacaoDaAtribuicaoNaoObservada` e publica
`unidade` (tela), `aberto_em` (siglas da árvore, `null` se não lidas),
`motivo` da leitura (`null` quando as unidades foram lidas) e
`requisicoes` efetivamente tentadas até a recusa. A presença de PR-P
na árvore não autoriza escolher PR-P se a tela publicou outra unidade.
Este diagnóstico é só leitura, não relaxa a validação nem habilita escrita.
Nenhum URL assinado, cookie ou senha é publicado. O subcódigo
`CharsetDaTelaNaoSuportado` indica MIME sem codificação suportada.

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
o desfecho é `NaoConfirmadoNaAtribuicao` com saída 1 e sem retry.
Uma recusa HTTP 5xx na releitura **interrompe** a resolução pelo índice,
sem buscar outra rota por tratá-la como link envelhecido. A falha tipada
preserva o status HTTP e a mensagem saneada da releitura quando a resposta
do POST não os forneceu, inclusive após erro de rede no POST.
`NaoConfirmadoNaAtribuicao` carrega também `requisicoes` com a contagem
real tentada, incluindo o POST único e o GET independente de prova;
a contagem não implica que a escrita foi aplicada.
**Releitura coincidente não converte recusa do POST em sucesso.** HTTP 403,
validação expressa da Infra (ainda que HTTP 200), demais respostas recusadas
um 302 com destino inesperado **ou HTML 200 genérico sem marcador
positivo de aceite** continuam coma escrita não confirmada (saída 1), mesmo
quando outro agente deixou a árvore no estado solicitado. A causa,
o status e a mensagem saneada da recusa expressa são preservados.
A observação UnB registrada no Todos @323eb01 mostra aceite por 302;
um HTML 200 genérico não identifica quem alterou a árvore. **Somente um 302 positivamente identificado** (ação, processo e origem)
permite confirmar o efeito depois da leitura. Um POST sem resposta é
**sempre escrita não confirmada (saída 1)**, mesmo que a árvore passe a mostrar
o usuário pedido ou a remoção: outro agente pode ter realizado a
mudança no intervalo. A leitura nova ainda registra
`atribuicao_relida` no erro (`None` = não observada, texto vazio =
remoção observada), como **possível resíduo e não prova de autoria**.
O erro original do envio fica preservado no erro tipado, sem retry.
**Não repita a confirmação de escrita não confirmada**, mesmo quando a
releitura mostra o estado pedido. Consulte o processo para verificar o possível resíduo, não para
transformar a tentativa inconclusiva em autorização de retry. O 302
reconhecido aponta para `arvore_visualizar` ou `principal` no mesmo
controlador, sempre com **`id_procedimento` explícito e igual ao processo
solicitado**, e com **`acao_origem=procedimento_atribuicao_cadastrar` explícita e única**. Um 302 sem essa origem não prova que a mudança relida veio deste POST; mesmo com processo e destino corretos, a saída é 1 após a leitura única. As duas ações são
genéricas sem identidade: a releitura coincidente pode ser efeito de
outro agente, inclusive na remoção. Sem ID, o resultado é escrita não confirmada
(saída 1), sem novo POST. A forma de `arvore_visualizar` com ID foi
observada no caso `ACEITE` do Todos @323eb01.
**Na CLI pública, `--confirmar` passa em qualquer das
[instalações públicas de treinamento](../../docs/interfaces/credenciais-de-treinamento.md)**
(antes de 09/11/2026, só na UnB, cuja resposta 302 com identidade está
documentada no oráculo Todos). A sessão privada
é lida antes dessa decisão; uma origem diferente, inclusive produção,
recebe `OrigemForaDoTreinamento` **antes de qualquer
requisição de rede**. A origem precisa ser idêntica, não apenas um host
parecido. A CLI chama o mesmo fluxo tipado de POST único e verificação por
nova leitura. O ensaio de 09/11/2026 em treinamento resultou em
**escrita não confirmada (saída 1)** após um único POST. A releitura observou o
login solicitado, mas não demonstrou que o POST provocou a alteração.
O PASS sintético não comprova causalidade no SEI.

O fluxo de escrita injetável utiliza o formulário servido como fonte do
payload, exatamente um POST sem retry e uma árvore nova para verificação.
Para ampliar essa habilitação a outras instalações, exige-se ensaio
seguro em treinamento, com prévia, confirmação única e registro da
atribuição ou remoção residual. A autorização se restringe a treinamento;
ordem prévia → confirmar uma vez → leitura de prova.

## Orçamento conhecido e pendências

Prévia: abertura do processo pelo índice/Controle, mais 1 GET da tela.
A escrita no fluxo de transporte acrescenta 1 POST e 1 GET de verificação,
sem repetir o POST mesmo sob falha de rede. O teste sintético a frio afirma
**5 GET + 1 POST = 6 requisições**, para atribuição e remoção; os caminhos
com índice local precisam de testes específicos.
Os testes de transporte sintético da prévia usam exatamente 4 GET e 0 POST a frio (3 da abertura e 1 da tela); barra ausente custa 3 GET e 0 POST. As recusas HTTP da prévia preservam status e mensagem saneada. Testes sintéticos do corpo do POST e da remoção usam 0 requisições:
`hdnInfraTipoPagina`, `sbmSalvar`, `selAtribuicao`, `hdnIdProtocolo`
nessa ordem. Em origens diferentes da UnB de treinamento a confirmação
continua fechada antes da rede, preservando o subcódigo existente.
RED de transporte: [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454), **4 testes novos FAIL, 1.689 PASS**.
Replay obrigatório das sete cassettes: **PASS** no [CI 42201836542](https://github.com/franklinbaldo/ies/actions/runs/42201836542), com 42 respostas classificadas, 28 formulários GET aceitos e 14 POSTs históricos sem prova de efeito; **0 HTTP real**. Prévia live na UnB: **PASS** no [ensaio 42205702494](https://github.com/franklinbaldo/ies/actions/runs/42205702494), com login de 5 requisições e 4 GET de prévia, **0 POST de atribuição e 0 resíduo**. Prévia na segunda instalação: **PASS** em GO 5.0.3 (`--remover`, 5 requisições; [evidência](../../docs/validacao-live/2026-11-06-processo-atribuir-go.md)). Em 09/11/2026 ocorreu um POST de atribuição em treinamento, com resultado **INCONCLUSIVO (saída 1)**; remoção live **NOT_RUN**.
RED da regressão de `principal` sem identidade: [CI 42204843683](https://github.com/franklinbaldo/ies/actions/runs/42204843683), **1 FAIL novo, 1.706 PASS, 23 ignored preexistentes**. Nenhuma escrita no SEI.

[Decisão](../../docs/decisoes/processo-atribuir.md),
[casos no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py),
[cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).

## Atualização de evidência externa — UnB, 09/11/2026

O [ensaio da CLI real](../../docs/validacao-live/2026-11-09-processo-atribuir-unb-post-incerto.md), incorporado na [PR #605](https://github.com/franklinbaldo/ies/pull/605), executou login (6 requisições), descoberta de processo elegível, uma prévia independente (3 requisições) e uma segunda prévia antes da confirmação (3 requisições). Ambas as prévias passaram sem POST. A confirmação realizou **um único POST**, terminou com **saída 1** após 5 requisições e publicou `atribuicao_relida=aluno2`. A operação inversa não foi chamada.

O estado relido é **um possível resíduo**, não prova de autoria: a resposta do envio não forneceu a identidade causal exigida por este contrato. A instalação é compartilhada e outra sessão pode produzir o mesmo estado. Não repetir a atribuição para tentar obter saída 0; qualquer eventual remoção exige decisão separada, leitura e prévia atuais. O registro privado `0600` foi preservado pelo executor de ensaio. Nenhuma operação em produção ocorreu.

**Evidências distintas:** IES↔anti-IES↔SQLite **PASS sintético anterior, NOT_RUN neste ensaio**; GET de atribuição↔cassettes **PASS restrito** para 28 formulários, POST histórico (14 respostas 302) **caracterizado sem confronto causal**; treinamento **prévia PASS, confirmação INCONCLUSIVA, remoção NOT_RUN**. Esta atualização não reclassifica os ensaios passados nem libera `--confirmar` para outras origens.
