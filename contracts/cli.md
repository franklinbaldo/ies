---
type: Contract
title: Contrato da CLI IES
description: Regras publicas provisórias da interface de linha de comando
---

# Contrato CLI do IES — fundação provisória

A interface CLI é a especificação primária; a implementação será guiada pelos
testes e pelas instalações reais.

## Invariantes

- Interface exclusivamente CLI, em português, orientada por substantivos e verbos de domínio.
- `ies <substantivo> NUMERO` lê; `ies <substantivo>` sem argumentos mostra ajuda.
- Não existe `agir`, `Action` ou `PageModel` como contrato público.
- `--json` e `--unidade` são globais; falhas operacionais são estruturadas.
- Códigos de saída: 0 sucesso, 1 falha, 2 uso inválido, 3 efeito incerto.
- Escritas exigem prévia, confirmação e verificação; efeito incerto nunca se repete automaticamente.
- Comportamentos são mapeados empiricamente em instalações reais do SEI.
- Esta superfície de demonstração não é integral: o inventário será aprovado comando a comando.

A [matriz de evidências](evidencias/README.md) relaciona operações
registradas pelo Todos a cassettes, mas **não equivale a aprovação dos
contratos desta CLI**. Contrato e evidência são fontes distintas.

## Independência de implementação

O contrato descreve somente comportamento e efeitos observáveis.
A decisão de usar REST, SOAP ou HTML pertence a um adaptador específico
e precisa de [evidência comparativa](../docs/arquitetura/0001-rotas-por-evidencia.md).
Nenhum método HTTP, resultado de uma API não habilitada ou mecanismo
legado do Todos faz parte do contrato público por pressuposição.

## Primeiro recorte, com acesso Web estritamente limitado

- `ies processo NUP`
- `ies documento NUMERO`
- `ies unidade listar-processos`
- `ies processo enviar NUP --para SIGLA`
- `ies documento assinar NUMERO`
- `ies documento salvar NUMERO [--versao N]` (prévia local, sem escrita remota)
- `ies status` (opção `--verificar-arquivos` confere metadados locais sem rede)
- `ies setup`
- `ies esquema`

## Contratos iniciais de leitura

- [processo.obter](cli/processo-obter.md): leitura do processo e projeções.
- [processo.buscar](cli/processo-buscar.md): NUP exato em resposta de pesquisa HTML privada, somente offline.
- [documento.obter](cli/documento-obter.md): conteúdo de um ou mais documentos.
- [documento.salvar](cli/documento-salvar.md): prévia local de cópia versionada, sem confirmação remota.
- [unidade.listar_processos](cli/unidade-listar-processos.md): caixa de processos.

O comando `ies unidade listar-processos` possui **leitura HTTP GET
restrita da caixa Web**, com sessão já existente e origem HTTPS
explicitamente autorizada, ou replay offline verificado. Não faz
login, POST ou paginação HTTP. Sem esses insumos retorna
`nao_implementado`.

`processo NUP`, `documento NUMERO` e a escrita remota continuam
como stubs. A prévia local de `documento salvar` não acessa o SEI. A autorização e a equivalência de outras rotas exigem
ensaios por cassette e por instalação.


## Diagnóstico opt-in do Controle de Processos Web (Refs #57)

`ies status --verificar-controle-web [--json]` realiza **um GET**
da URL assinada do Controle de Processos configurada em arquivos
privados, aplicando as restrições existentes de HTTPS, mesma origem,
cookies e ausência de redirects. A operação é explicitamente
solicitada: `ies status` sem opções,
`--verificar-arquivos` e `--verificar-sessao-local`
continuam **sem rede**; opções de diagnóstico são mutuamente
exclusivas. Um GET do SEI pode registrar visualização/recebimento:
usar apenas quando essa leitura estiver autorizada e seus efeitos
forem conhecidos para a instalação.

HTTP 200 isolado, página de login, HTML genérico ou uma página
sem identificação inequívoca da unidade **não são sucesso**.
A resposta precisa conter Controle de Processos reconhecível,
tabelas de Recebidos/Gerados e `lnkInfraUnidade` único.
O diagnóstico não publica números de processos, HTML, cookies,
hashes, URLs nem detalhes de outros usuários, somente
`controle_web: observado`, `unidade_exibida`,
`conexao: get_realizado` e
`autenticacao_remota: nao_auditada`. Erros
`configuracao_invalida`, `transporte_invalido`,
`controle_web_nao_reconhecido` e
`identidade_unidade_nao_observada` são variantes
Rust específicas sem classificar mensagens por regex.

Isso não é login automático, teste de permissões para operações
protegidas, prova de cobertura/paginação nem certificação de
autenticação do usuário. Testes live nas instalações autorizadas
seguem exigindo evidência independente; um GET 200 não comprova
o restante da API.

## Busca textual offline: validação da entrada bruta (Refs #61)

Em `ies replay CENARIO --documento --termo-documento TERMO`, a entrada
original é validada **antes** de aparar espaços externos. CR, LF, tab e
outros caracteres de controle, mesmo nas extremidades, retornam a variante
Rust `ErroBuscaDocumento::TermoComControle`, subcódigo
`termo_de_documento_contem_controle` e saída 2. Espaços comuns externos
continuam tolerados; termo vazio continua tendo erro específico.
Não alterar a busca por Unicode nem tratar ausência na cassette como
inexistência no SEI. Sem GET, POST ou validação live.

## Unidade esperada no diagnóstico Web (Refs #57, #56)

`ies --unidade SIGLA status --verificar-controle-web --json` confere o
rótulo da unidade **efetivamente publicado** no Controle de Processos
antes de relatar sucesso. A opção global `--unidade` não pode ser
ignorada por este diagnóstico com rede. Em caso de diferença literal,
retornar `unidade_exibida_diferente_da_solicitada` sem imprimir o
rótulo observado nem dados da sessão. Sem `--unidade`, conservar o
resultado atual. A comparação é de igualdade exata, sem heurística
para siglas, normalização ou troca automática de unidade.

A operação continua limitada ao GET já solicitado (zero pedidos
adicionais), sem login ou POST. Este teste de identidade da tela não
comprova sessão autenticada, permissões ou cobertura de paginação.
