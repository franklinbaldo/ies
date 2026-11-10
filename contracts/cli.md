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
- Códigos de saída: 0 sucesso (escrita confirmada por releitura), 1 falha (inclusive escrita não confirmada), 2 uso inválido.
- Escritas exigem prévia, confirmação e verificação; escrita não confirmada nunca se repete automaticamente.
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

- `ies processo comentarios NUP` e `ies processo comentar NUP --texto T [--confirmar]` (contrato [comentários](cli/processo-comentarios.md); núcleo offline na PR Draft #268, CLI Web ainda pendente)
- `ies bloco listar [--tipo assinatura|interno|reuniao]` e `ies bloco N`
  (leitura Web por menu observado; contratos
  [listar](cli/bloco-listar.md) e [obter](cli/bloco-obter.md); #256).
- `ies bloco incluir BLOCO --processo NUP | --documento DOC [--confirmar]`
  (contrato unificado de inclusão em bloco interno/reunião ou assinatura,
  [bloco.incluir](cli/bloco-incluir.md), #381 e #343; ainda não implementado).


- `ies processo NUP`
- `ies processo buscar TERMO [--pagina N]` (pesquisa Web: GET + GET + POST;
  `--pagina N` acrescenta um POST de `navegar()`)
- `ies documento NUMERO [--versao N] [--saida DIR]` (versão N pela lista
  da barra, Refs #88; com `--saida`, foto `DOC.vN.html` do interno
  ou arquivo original do externo, idempotentes, #207)
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
- `ies processo marcar NUP --marcador ID#NOME|--retirar ID#NOME [--texto T] [--confirmar]` (prévia e idempotência pela leitura Web; escrita ainda fechada, [contrato](cli/processo-marcar.md))
- `ies processo sobrestar NUP --motivo TEXTO [--confirmar]` e
  `ies processo retomar NUP [--confirmar]` ([contrato parcial](cli/processo-sobrestar.md):
  prévia Web; `--confirmar` com uma escrita e verificação por releitura, saída 1 se não confirmada)
- `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
  ([contrato](cli/processo-enviar.md))
- `ies processo ciente NUP [--confirmar]` (ciência Web: um GET mutador,
  releitura da árvore e da lista; [contrato](cli/processo-ciente.md); #271)
- `ies documento ciente NUMERO [--confirmar]` (GET único da ciência e verificação pela Lista de Ciências do documento; [contrato](cli/documento-ciente.md); #271)
- `ies documento cancelar NUMERO --motivo TEXTO [--confirmar]` (prévia e cancelamento Web irreversível, um POST com verificação pela árvore; [contrato](cli/documento-cancelar.md))
- `ies documento assinar NUMERO [--cargo TEXTO] [--confirmar]` (assinatura
  com a senha do login, prévia e verificação pela caneta da árvore;
  [contrato](cli/documento-assinar.md))
- `ies documento salvar NUMERO [--versao N]` (prévia local, sem escrita remota)
- `ies indice caminho` (caminho do SQLite local, sem abrir banco ou consultar SEI)
- `ies unidade listar`/`pesquisar` (lista da lupa/SQLite; [contratos](cli/unidade-listar.md))
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

## Configurar caminhos locais

`ies setup --config-file ARQUIVO --url-controle-file ARQUIVO
--cookie-file ARQUIVO --origem-file ARQUIVO` exige as quatro opções
no clap. São caminhos, nunca conteúdo de sessão. `--json` e
`--unidade` são globais; setup não verifica unidade nem faz rede.

O destino da configuração e os três caminhos devem ser absolutos.
Os três arquivos devem existir e, no Unix, ter permissões privadas; o setup
confere metadados, não testa a conexão. Grava a configuração local
com criação exclusiva (não sobrescreve) e permissão 0600 no Unix. Sucesso (0) em JSON:
`{"configuracao":"gravada","conexao":"nao_verificada"}`.
Falha operacional usa `fonte_invalida`, mensagem saneada
e saída 1; opção obrigatória ausente é erro do clap (2).

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
  detalhe; sem `--confirmar`, prévia sem POST de salvar; escrita não confirmada sai com 1.
- [unidade.trocar](cli/unidade-trocar.md): troca da unidade da sessão; prévia
  sem POST, escrita não confirmada sai com 1; vale para todos os agentes da sessão.
- [indice.caminho](cli/indice-caminho.md): descoberta local do SQLite, sem rede.
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

## Replay de cassettes: fora da CLI pública

Não há `ies replay`: o subcomando é erro de uso (saída 2). O binário de
produção não lê cassettes nem liga `zip` e `sha1`. O replay do acervo
público são os testes `#[ignore]` `*::acervo*` do crate, que passam as
respostas gravadas pelos **mesmos leitores** dos comandos, offline; o
[script `scripts/replay`](../scripts/replay) os lista (sem argumentos) e os
executa (`scripts/replay FILTRO`). Como usar: [cassettes](../docs/cassettes.md#replay-do-acervo);
por quê e o que cada opção antiga virou:
[decisão](../docs/decisoes/acervo/replay-fora-da-cli.md).

## Recusas HTTP e mensagem do SEI

As recusas de resposta do GET (uma variante por fato) e o status e a
mensagem do SEI nas recusas HTTP estão em
[recusas HTTP](cli/recusas-http.md).

## Registro de andamento

[Contrato experimental do comando processo andamento](cli/processo-andamento.md): prévia Web, POST único e confirmação por histórico relido. Não é capacidade aprovada até CI, replay e live.

## Partes do contrato

- [Leituras paginadas](cli/leituras-paginadas.md): página observada por padrão e --pagina N nas leituras do Controle.
- [Diagnóstico do Controle](cli/diagnostico-controle.md): diagnóstico opt-in do Controle de Processos Web e unidade esperada.
- [Transporte HTTPS e relogin](cli/transporte-https.md): um cliente HTTPS com o proxy do ambiente em todos os comandos e o relogin único nas leituras.
- [Escritas do processo](cli/escritas-do-processo.md): resumo de concluir, reabrir e enviar o processo, com os subcódigos próprios.
