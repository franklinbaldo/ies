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
  `--unidade` é pré-condição conferida na tela, nunca um rótulo ecoado.
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
- `ies processo buscar TERMO [--pagina N]` (pesquisa Web: GET + GET + POST;
  `--pagina N` acrescenta um POST de `navegar()`)
- `ies documento NUMERO`
- `ies unidade listar-processos [--pagina N]` (visão detalhada com colunas;
  a sessão fica na D, sem restaurar; decisões do proprietário, 02/11/2026)
- `ies unidade recebidos [--pagina N]` e `ies unidade gerados [--pagina N]`
  (uma lista; decisão do proprietário, 02/11/2026)
- `ies unidade colunas` (Nível de detalhe, só leitura) e
  `ies unidade colunas --definir todas|LISTA [--confirmar]` (escrita da
  preferência de colunas, com prévia, confirmação e verificação)
- `ies processo enviar NUP --para SIGLA`
- `ies documento assinar NUMERO`
- `ies documento salvar NUMERO [--versao N]` (prévia local, sem escrita remota)
- `ies status` (opção `--verificar-arquivos` confere metadados locais sem rede)
- `ies setup`
- `ies login [--forcar]` (login Web na instalação configurada; grava a sessão
  nos arquivos privados)
- `ies esquema`

## Contratos iniciais de leitura

- [processo.obter](cli/processo-obter.md): leitura do processo e projeções.
- [processo.buscar](cli/processo-buscar.md): pesquisa Web por protocolo com a
  sessão de `ies login` (processo único, lista com total e faixa ou "sem
  resultado").
- [documento.obter](cli/documento-obter.md): conteúdo de um ou mais documentos.
- [documento.salvar](cli/documento-salvar.md): prévia local de cópia versionada, sem confirmação remota.
- [unidade.listar_processos](cli/unidade-listar-processos.md): caixa de processos;
  [visão detalhada](cli/unidade-listar-processos-visao-detalhada.md),
  [listas Recebidos e Gerados](cli/unidade-recebidos-gerados.md) com forma única
  e [colunas do Nível de detalhe](cli/unidade-colunas.md).
- [unidade.definir_colunas](cli/unidade-definir-colunas.md): escrita do Nível de
  detalhe; sem `--confirmar`, prévia sem POST de salvar; efeito incerto sai com 3.
- [sessao.login](cli/sessao-login.md): `ies login`, credenciais em arquivo
  `0600`, um GET de conferência da sessão gravada, redirecionamentos só no
  host da instalação e gravação atômica por geração, sob lock.

Os comandos `ies unidade listar-processos`, `recebidos`, `gerados` e
`colunas` têm **leitura HTTP restrita da caixa Web**, com sessão já existente
(gravada por `ies login` ou à mão) e origem HTTPS explicitamente autorizada,
ou replay offline verificado. Não fazem login automático (sessão vencida é
falha tipada; relogar é `ies login`) nem escrita; a única escrita é `colunas --definir … --confirmar`. Os POSTs previstos são os de `--pagina N` (abaixo)
e os da **troca de visão** R↔D, sem volta: a sessão fica na visão que o
comando precisou ([visão detalhada](cli/unidade-listar-processos-visao-detalhada.md)).
Sem esses insumos retorna `nao_implementado`.

`ies processo buscar TERMO` usa a mesma sessão e as mesmas travas: segue só o
link `protocolo_pesquisar` publicado pelo Controle; seus POSTs são o da
pesquisa e, com `--pagina N`, um de `navegar()`, sem restaurar a página ou o
filtro lembrados ([contrato](cli/processo-buscar.md)).

## Leituras paginadas: página observada por padrão (Refs #57)

Invariante de **toda** leitura paginada da CLI:

- **Sem opção**, a leitura usa só a tela já obtida (GET ou arquivo),
  sem POST de paginação, e devolve a página que a tela mostra com o
  `total_declarado` e a `faixa_observada` (`inicio`/`fim` das posições
  lidas) de cada lista. A faixa não diz "primeira página": o servidor
  pode lembrar a última página pedida na sessão.
- A saída **nunca** publica `completa`, `completude` ou afirma lista
  integral ou snapshot; quem consome compara faixa e total.
- O opt-in é único e tem o mesmo nome em todos os comandos:
  **`--pagina N`** (inteiro ≥ 1), que pede exatamente a página N com
  o mínimo de requisições. Não há "todas as páginas".
- N fora do intervalo calculável pelo total e pelo tamanho de página
  observados é erro tipado **antes** do envio da página (se o tamanho da
  página só aparece depois de uma troca de visão, depois dela; a sessão
  fica na visão trocada). A página obtida confere unidade, visualização e
  critérios da tela de origem; não há retry. O efeito é
  `PaginaOuFiltroLembrado`: a leitura seguinte pode mostrar a página N, e a
  faixa publicada diz qual é. **Nenhuma leitura restaura** a visão ou a
  página que moveu (decisão do proprietário, 02/11/2026, na
  [decisão da operação](../docs/decisoes/unidade-listar-processos.md)):
  com a sessão própria do IES, o efeito residual é cosmético (num SEI 5.x
  um login novo da mesma conta herda a última visão) e a página lembrada
  aparece em `faixa_observada` da leitura seguinte.

Contradições reais (faixa inválida ou não contígua, total ausente,
unidade ou contexto divergente) continuam recusas tipadas.

Continuam como stubs `documento DOC` (obtenção Web), `documento buscar`,
`documento salvar --confirmar` remoto, `processo enviar`, `processo
concluir` e `documento assinar`. `processo NUP` lê pela árvore
([contrato](cli/processo-obter-arvore.md)) e `unidade colunas --definir
--confirmar` é a escrita remota implementada
([contrato](cli/unidade-definir-colunas.md)). A prévia local de `documento salvar` não acessa o SEI. A autorização e a equivalência de outras rotas exigem
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

## Recusas de resposta do GET com variante por fato (Refs #57)

O antigo subcódigo único `resposta_inesperada` foi separado, porque o
ensaio live em Goiás (02/11/2026) não pôde dizer qual verificação
falhou. Toda resposta do GET Web passa pelas mesmas checagens, na ordem:

| Fato observado | Variante Rust | Subcódigo |
| --- | --- | --- |
| Status HTTP 5xx | `StatusHttpDeErroDoServidor` | `status_http_de_erro_do_servidor` |
| Status HTTP 429 | `StatusHttpDeLimiteDeRequisicoes` | `status_http_de_limite_de_requisicoes` |
| Outro status diferente de 200 (fora 3xx/401/403) | `StatusHttpDiferenteDe200` | `status_http_diferente_de_200` |
| `Content-Type` principal diferente de `text/html` | `TipoDeConteudoNaoHtml` | `tipo_de_conteudo_nao_html` |
| Corpo não decodifica no charset declarado (ou UTF-8) | `CorpoNaoDecodificavelNoCharsetDeclarado` | `corpo_nao_decodificavel_no_charset_declarado` |

3xx, 401/403, limite de tamanho e tela de login mantêm as variantes
existentes. Nenhuma das recusas autoriza nova tentativa automática.

## Status e mensagem do SEI nas recusas HTTP (Refs #57)

Quem lê a recusa precisa saber **o que o SEI respondeu**, não só a faixa
do status. Em GO (02/11/2026), dois 5xx no POST de paginação não
puderam ser explicados. Os subcódigos acima continuam **exatamente os
mesmos** e são a classificação; o status e a mensagem são apresentação
e nunca decidem tratamento.

Todo comando que faz o GET do Controle ou o POST do paginador (`status
--verificar-controle-web`, `unidade atual`, `listar-processos`,
`recebidos`/`gerados [--pagina N]`, `colunas`), inclusive nos POSTs da troca de visão, publica,
no JSON da falha `fonte_invalida`, dois campos a mais:

| Campo | Quando | Conteúdo |
| --- | --- | --- |
| `status_http` | 3xx, 401, 403, 429, 5xx e outro status diferente de 200 | número do status |
| `mensagem_do_servidor` | só se o corpo da recusa (fora 3xx) rendeu texto | texto saneado |

Recusas locais (configuração, arquivos privados, MIME, charset,
tamanho, login) não ganham esses campos. A saída humana acrescenta
`(HTTP N: mensagem)` à linha da falha.

Leitura do corpo da recusa: no máximo **64 KiB**, lidos em fluxo (o
resto nunca é lido); uma única tentativa, sem retry nem redirect. 3xx
**não** tem o corpo lido; 302 do POST continua dado da operação. Falha
na leitura do corpo não mascara o status: só omite a mensagem.
Decodificação: charset ISO-8859-1/latin1/windows-1252 declarado vira
ISO-8859-1; senão UTF-8, e bytes inválidos caem para ISO-8859-1.

Extração, com `tl` e `html_atributos` (sem regex): se o `Content-Type`
é HTML, o texto do `#txaInfraValidacao` não vazio
([validação da Infra](../docs/como-o-sei-funciona/validacao-da-infra.md)),
senão o de `#divInfraExcecao` (a tela de exceção do SEI, que
as páginas reais testam por `getElementById('divInfraExcecao')`), senão
o de `div.infraExcecao` ou `div.infraMensagemErro`, senão `<title>` e o
texto visível do `<body>` (sem `script`, `style`, `noscript`,
`template`); entidades básicas decodificadas. Fora de HTML, o corpo é
texto puro.

Saneamento, por varredura explícita: controles viram espaço, espaços
são colapsados; URLs `http(s)://…` → `<url>`; valor de `infra_hash=` e
`PHPSESSID=` → `<redigido>`; sequência de 32 ou mais dígitos
hexadecimais → `<hex>`; corte em 500 caracteres, na fronteira de
caractere, com `…`. Fora isso o texto sai **como o SEI o enviou** e
pode conter informação do servidor (caminhos, nomes de classe, SQL):
não o republique sem revisar.

Validação da Infra num POST com HTTP 200 (troca de visão, `--pagina N`,
`colunas --definir`, pesquisa): `#txaInfraValidacao` não vazio é
`validacao_recusada_pelo_sei`, com ou sem o marcador de resultado da
operação; publica só `mensagem_do_servidor` (sem `status_http`, pois o
status foi 200) e a linha humana ganha `(SEI: mensagem)`. Em `colunas
--definir` o efeito fica incerto e segue para a leitura de verificação.
Textarea vazio ou ausente não muda a classificação.

```json
{"codigo":"fonte_invalida","subcodigo":"status_http_de_erro_do_servidor",
 "erro":"o SEI respondeu com erro do servidor (5xx)","retentavel":false,
 "status_http":500,"mensagem_do_servidor":"Erro acessando o banco de dados."}
```

401/403 continuam `sessao_nao_autorizada`, agora com `status_http` e a
mensagem; 3xx continua `redirecionamento_recusado`, com `status_http`.
O 403 com a página de bloqueio de segurança da instalação (`<title>Acesso
Negado</title>` e "bloqueada por nossa política de segurança" no texto
visível; visto em GO), ou a do Cloudflare (`<title>Attention Required! |
Cloudflare</title>` e "Sorry, you have been blocked"; visto na PEN2 e na PEN3), é
`bloqueio_de_seguranca_da_instalacao`, com
`status_http` 403 e a mensagem saneada, que traz o código de bloqueio. A
sessão não venceu: `ies login` não reloga por ele, e nada é repetido.

## Transporte HTTPS: um cliente, proxy do ambiente em todos os comandos

Todo pedido de rede do IES (o GET do Controle, os POSTs dos comandos e o
`ies login`) passa por **um único cliente HTTPS**
(`src/transporte/cliente_https.rs`): HTTP/1.1 sobre `native-tls`, uma
conexão por pedido, `Connection: close`, `Accept-Encoding: identity`,
`User-Agent: ies/<versão>`. Antes, só o login atravessava proxy; os
demais comandos conectavam direto e falhavam em rede com saída só por
proxy.

- **Proxy:** as mesmas regras do
  [login](cli/sessao-login.md#proxy-do-ambiente), agora para todos os
  comandos: a primeira não vazia de `https_proxy`, `HTTPS_PROXY`,
  `all_proxy`, `ALL_PROXY` (`http_proxy` não vale); exceções em
  `no_proxy`, senão `NO_PROXY` (`*`, `.sufixo`, host exato), conferidas
  antes do endereço; só `http://`, credencial apenas em
  `Proxy-Authorization: Basic` do `CONNECT` (nunca no pedido ao SEI). Um
  `CONNECT host:443` por pedido; o TLS segue com o host do SEI.
- **Falhas do proxy:** os subcódigos `endereco_de_proxy_invalido`,
  `proxy_com_esquema_nao_suportado`, `proxy_inacessivel`,
  `resposta_do_proxy_nao_interpretavel` e `proxy_recusou_o_tunel`, sem
  `status_http` (o SEI não respondeu). Nos comandos fora do login saem em
  `fonte_invalida`; o campo `status_do_proxy` continua só no `ies login`.
- **Travas dos comandos, inalteradas:** só HTTPS; origem da instalação
  conferida antes de abrir o cookie; **nenhum** redirecionamento seguido
  (3xx é `redirecionamento_recusado`, com `status_http`, corpo não lido;
  302 do POST continua dado da operação); 30 s de tempo limite de conexão,
  leitura e escrita; HTML limitado a **8 MB** durante o recebimento
  (`Content-Length` maior é recusado antes da leitura; `chunked` e corpo
  até o fim da conexão também); corpo da recusa lido até **64 KiB**;
  `Content-Type` e charset como antes; tela de login detectada; uma única
  tentativa, sem retry.
- **Login:** mesmo cliente; lê o corpo inteiro (8 MB), guarda todos os
  `Set-Cookie` e mantém o próprio intérprete de redirecionamentos, que
  não vale para outros comandos.

Testado offline com proxy falso local (`TcpListener`) e camada TLS
substituída pela identidade: `CONNECT`, pedido no túnel, ausência de
segundo pedido, `NO_PROXY`, 3xx, limites e subcódigos. O TLS real
através do túnel não tem teste automatizado; validação live: `NOT_RUN`.

## Concluir o processo (Refs #172)

`ies processo concluir NUP [--confirmar]` é a primeira escrita de estado do
processo ([contrato](cli/processo-concluir.md)): prévia só com a abertura;
uma escrita pela forma que a barra publica; verificação por `aberto_em`
numa leitura nova. Subcódigos próprios: `situacao_do_processo_nao_lida_antes_de_concluir`,
`unidade_da_sessao_ausente_da_situacao`, `conclusao_nao_publicada_na_barra`,
os da barra (`formas_de_concluir_repetidas_na_barra`,
`endereco_da_conclusao_nao_reconhecido`, `endereco_da_tela_do_no_nao_reconhecido`, …),
os da tela (`campo_nao_previsto_na_tela_de_conclusao`,
`opcao_somente_concluir_nao_marcada`, `tela_de_conclusao_com_reabertura_preenchida`, …),
os da função (`funcao_de_concluir_ausente_na_tela_do_no`, …),
`conclusao_nao_aplicada` (`escrita_nao_aplicada`, saída 1) e
`efeito_incerto_na_conclusao` (`efeito_incerto`, saída 3).

## Reabrir o processo (Refs #172)

`ies processo reabrir NUP [--confirmar]` é o espelho de `processo concluir`
([contrato](cli/processo-reabrir.md)): prévia só com a abertura; o GET da
tela do nó e o GET da URL de reabrir lida nela (a escrita); verificação
por `aberto_em` numa leitura nova. Subcódigos próprios:
`situacao_do_processo_nao_lida_antes_de_reabrir`,
`unidade_da_sessao_ja_na_situacao`, `reabertura_nao_publicada_na_barra`,
`funcoes_de_reabrir_repetidas_na_barra`, os da tela do nó
(`endereco_de_reabrir_ausente_na_tela_do_no`,
`enderecos_de_reabrir_repetidos_na_tela_do_no`, …),
`tela_do_no_redirecionada`, `redirecionamento_na_reabertura` (resposta),
`reabertura_nao_aplicada` (`escrita_nao_aplicada`, saída 1) e
`efeito_incerto_na_reabertura` (`efeito_incerto`, saída 3).
