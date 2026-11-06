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
Nenhum URL assinado, cookie ou senha é publicado. O subcódigo
`charset_da_tela_nao_suportado` indica MIME sem codificação suportada.

## Estados desta fatia

A **prévia sem --confirmar** é leitura Web com contagem de requisições
(até a abertura da tela de atribuição); devolve atual, solicitado e valor
da opção, sem POST. Também deriva o payload pelo formulário servido
(botão Salvar, ocultos originais e `selAtribuicao` exato) e recusa método,
charset ou codificação incompatíveis, sem escrever. ID numérico que aparece
para dois logins diferentes é recusado: não há identidade unívoca.
**--confirmar está temporariamente bloqueado** por
`escrita_de_atribuicao_ainda_nao_verificada`, sem acessar o SEI: a
integração de escrita só será habilitada quando houver teste de
POST único e releitura.

A escrita definitiva deverá usar o formulário servido como fonte do
payload, exatamente um POST sem retry, verificação por uma árvore nova
e saída 3 em efeito incerto. A autorização se restringe às instalações
de treinamento; ordem prévia → confirmar uma vez → leitura de prova.

## Orçamento conhecido e pendências

Prévia: abertura do processo pelo índice/Controle, mais 1 GET da tela.
A escrita futura acrescentará 1 POST e 1 leitura da árvore; os testes
integrados ainda precisam afirmar os totais por caminho de abertura.
Testes sintéticos do corpo do POST e da remoção usam 0 requisições:
`hdnInfraTipoPagina`, `sbmSalvar`, `selAtribuicao`, `hdnIdProtocolo`
nessa ordem. `--confirmar` continua fechado antes da sessão. Replay do
acervo e live: NOT_RUN nesta fatia, sem escrita.

[Decisão](../../docs/decisoes/processo-atribuir.md),
[casos no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py),
[cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).