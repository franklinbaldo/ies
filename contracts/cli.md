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
- `ies documento NUMERO [--versao N] [--saida DIR] [--sem-copia]` (versão N
  pela lista da barra; cópia de trabalho `DOC.vN.html`, Refs #88)
- `ies documento buscar TERMO [--pagina N]` (a mesma pesquisa Web, em
  documentos: GET + GET + POST; `--pagina N` acrescenta um POST)
- `ies unidade listar-processos [--pagina N]` (visão detalhada com colunas;
  a sessão fica na D, sem restaurar; decisões do proprietário, 02/11/2026)
- `ies unidade recebidos [--pagina N]` e `ies unidade gerados [--pagina N]`
  (uma lista; decisão do proprietário, 02/11/2026)
- `ies unidade colunas` (Nível de detalhe, só leitura) e
  `ies unidade colunas --definir todas|LISTA [--confirmar]` (escrita da
  preferência de colunas, com prévia, confirmação e verificação)
- `ies unidade trocar SIGLA [--confirmar]` (troca a unidade da sessão para
  toda a conta, com prévia e verificação pelo Controle novo;
  [contrato](cli/unidade-trocar.md))
- `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
  ([contrato](cli/processo-enviar.md))
- `ies documento assinar NUMERO [--cargo TEXTO] [--confirmar]` (assinatura
  com a senha do login, prévia e verificação pela caneta da árvore;
  [contrato](cli/documento-assinar.md))
- `ies documento salvar NUMERO [--versao N]` (prévia local, sem escrita remota)
- `ies status` (opção `--verificar-arquivos` confere metadados locais sem rede)
- `ies setup`

**Configuração local do setup/status (#212).** Com `IES_CONFIG_FILE`
definido, o IES não recorre aos caminhos individuais quando o arquivo
é recusado. `arquivo_configuracao_nao_pode_ser_lido` permanece para
ausência, tipo não regular, tamanho excessivo e erro de leitura; as
recusas novas são `arquivo_configuracao_link_simbolico`,
`arquivo_configuracao_permissoes_inseguras`,
`arquivo_configuracao_json_invalido` e
`arquivo_configuracao_campo_invalido`. Os campos do JSON devem
conter caminhos absolutos não vazios; não há leitura remota nem
exposição do caminho recusado. Ver [erros tipados](../docs/erros-tipados.md).

- `ies login [--forcar]` (login Web na instalação configurada; grava a sessão
  nos arquivos privados)
- `ies esquema`

## Contratos iniciais de leitura

- [processo.obter](cli/processo-obter.md): leitura do processo e projeções.
- [processo.buscar](cli/processo-buscar.md): pesquisa Web por protocolo com a
  sessão de `ies login` (processo único, lista com total e faixa ou "sem
  resultado").
- [documento.obter](cli/documento-obter.md): conteúdo de um ou mais documentos;
  [versões e cópia de trabalho](cli/documento-obter-versoes.md).
  O externo usa o [GET binário](cli/documento-obter-externo.md), com
  subcódigos próprios (`tipo_de_anexo_nao_permitido`,
  `anexo_acima_do_limite`, `anexo_recusado_pelo_sei` e afins).
- [documento.buscar](cli/documento-buscar.md): pesquisa Web em documentos pelo número SEI.
- [documento.salvar](cli/documento-salvar.md): prévia local de cópia versionada, sem confirmação remota.
- [unidade.listar_processos](cli/unidade-listar-processos.md): caixa de processos;
  [visão detalhada](cli/unidade-listar-processos-visao-detalhada.md),
  [listas Recebidos e Gerados](cli/unidade-recebidos-gerados.md) com forma única
  e [colunas do Nível de detalhe](cli/unidade-colunas.md).
- [unidade.definir_colunas](cli/unidade-definir-colunas.md): escrita do Nível de
  detalhe; sem `--confirmar`, prévia sem POST de salvar; efeito incerto sai com 3.
- [unidade.trocar](cli/unidade-trocar.md): troca da unidade da sessão; prévia
  sem POST, efeito incerto sai com 3; vale para todos os agentes da sessão.
- [sessao.login](cli/sessao-login.md): `ies login`, credenciais em arquivo
  `0600`, um GET de conferência da sessão gravada, redirecionamentos só no
  host da instalação e gravação atômica por geração, sob lock.

Os comandos `ies unidade listar-processos`, `recebidos`, `gerados` e
`colunas` têm **leitura HTTP restrita da caixa Web**, com sessão já existente
(gravada por `ies login` ou à mão) e origem HTTPS explicitamente autorizada.
Sessão vencida é falha tipada; com o arquivo
de credenciais do `ies login`, a leitura reloga **uma vez** e recomeça
([relogin único nas leituras](cli/sessao-login.md#relogin-único-nas-leituras)).
Não fazem escrita; a única escrita é `colunas --definir … --confirmar`, que
nunca reloga. Os POSTs previstos são os de `--pagina N` (abaixo)
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

`documento DOC` lê documentos internos e externos pela Web (#60 fatias 1 a 3;
[contrato](cli/documento-obter-web.md)). Continua como stub
`documento salvar --confirmar` remoto; `processo enviar`
([contrato](cli/processo-enviar.md)) e `documento assinar`
([contrato](cli/documento-assinar.md)) escrevem com prévia e verificação. `processo concluir` escreve com prévia e verificação
([contrato](cli/processo-concluir.md)); `documento DOC --versao N` está
implementado ([contrato](cli/documento-obter-versoes.md)). `processo NUP`
lê pela árvore ([contrato](cli/processo-obter-arvore.md)) e `unidade
colunas --definir --confirmar` também escreve
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

## Replay de cassettes: fora da CLI pública

Não há `ies replay`: o subcomando é erro de uso (saída 2). O binário de
produção não lê cassettes nem liga `zip` e `sha1`. O replay do acervo
público são os testes `#[ignore]` `*::acervo*` do crate, que passam as
respostas gravadas pelos **mesmos leitores** dos comandos, offline; o
[script `scripts/replay`](../scripts/replay) os lista (sem argumentos) e os
executa (`scripts/replay FILTRO`). Como usar: [cassettes](../docs/cassettes.md#replay-do-acervo);
por quê e o que cada opção antiga virou:
[decisão](../docs/decisoes/acervo/replay-fora-da-cli.md).

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
  o 302 do GET do Controle para o login da instalação é
  `sessao_encerrada_rode_ies_login`; 302 do POST continua dado da operação); 30 s de tempo limite de conexão,
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

## Relogin único nas leituras (Refs #200)

`ies processo NUP`, `processo buscar`, `documento DOC`, `documento buscar`,
`unidade listar-processos`, `recebidos`, `gerados`, `atual` e `colunas` sem
`--definir` relogam no máximo uma vez, só depois de
`pagina_de_login_recebida_no_controle` ou `sessao_encerrada_rode_ies_login`,
e recomeçam do início uma vez. Antes de logar, sob o lock do `ies login`,
conferem a geração gravada: se outro processo já relogou, usam a sessão
nova sem login. A saída ganha `relogin: true` (ausente sem login deste
processo) e `requisicoes` soma a primeira tentativa, o login e a releitura.
Escrita e prévia de escrita nunca relogam. Regras, orçamento e saídas:
[relogin único nas leituras](cli/sessao-login.md#relogin-único-nas-leituras).

## Recusas HTTP e mensagem do SEI

As recusas de resposta do GET (uma variante por fato) e o status e a
mensagem do SEI nas recusas HTTP estão em
[recusas HTTP](cli/recusas-http.md).

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

## Enviar o processo (Refs #173)

`ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
([contrato](cli/processo-enviar.md)): prévia com a abertura, a tela
"Enviar Processo" e a consulta do autocomplete que ela declara (POST de
leitura, nenhum de escrita); um POST e verificação pelo destino e pela
unidade da sessão em `aberto_em`. Uso inválido (2):
`sigla_de_destino_invalida`, `mais_de_um_destino_nao_suportado`.
Subcódigos próprios: `situacao_do_processo_nao_lida_antes_de_enviar`,
`processo_nao_aberto_na_unidade_da_sessao`, `envio_nao_publicado_na_barra`,
`links_de_enviar_repetidos_na_barra`, os da tela
(`campo_nao_previsto_na_tela_de_envio`, `tela_de_envio_com_opcoes_preenchidas`,
`autocompletar_de_unidade_nao_declarado`, …), os da consulta
(`resposta_do_autocompletar_nao_xml`, …), `destino_nao_oferecido_pelo_sei`
(`oferecidos`), `varios_destinos_com_a_sigla` (`candidatos`),
`processo_ja_aberto_no_destino`, `envio_nao_aplicado`
(`escrita_nao_aplicada`, 1), `situacao_da_origem_diferente_da_pedida_apos_envio`
(`escrita_diferente_da_solicitada`, 1) e `efeito_incerto_no_envio`
(`efeito_incerto`, 3).
