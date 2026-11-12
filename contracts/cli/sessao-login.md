---
type: CommandContract
title: Login Web na instalação configurada
operation: sessao.login
command: ies login [--forcar]
effect: read
implementation: partial
source_commit: 6127bd38b9182108c9b0922d6796493e338e5f05
---

# Login Web na instalação configurada

`ies login` autentica na interface Web da instalação já configurada
(arquivo de origem, a `Instalacao`) e grava a sessão nos **mesmos três
arquivos privados `0600`** que os demais comandos leem
(`IES_ORIGEM_FILE`, `IES_COOKIE_FILE`, `IES_URL_CONTROLE_FILE` ou a
configuração do `ies setup`). Não há escrita de domínio no SEI; o efeito
é criar uma sessão (estado da sessão no servidor) e regravar os
arquivos locais de cookie e URL do Controle. A origem é lida, nunca
reescrita.

## Entrada

- Credenciais só por arquivo privado: caminho em `IES_CREDENCIAIS_FILE`,
  arquivo regular, sem link simbólico, `0600`, até 4 KiB, UTF-8. Nunca
  por argumento. Formato, uma chave por linha, cada uma exatamente uma
  vez, linhas vazias ignoradas:

  ```text
  usuario=<login>
  senha=<senha, tudo depois do primeiro "=", sem aparar>
  orgao=<sigla como a tela de login a mostra no selOrgao>
  ```

  `usuario` e `orgao` são aparados; `senha` sai como escrita. Linha sem
  chave conhecida, chave repetida, ausente ou vazia é recusa tipada.
  O arquivo só é lido quando um login é necessário.
- `--forcar`: faz um novo login mesmo com sessão válida.
- `--unidade SIGLA` (global): pré-condição conferida no Controle obtido.
  Diferença literal falha com `UnidadeExibidaDiferenteDaSolicitada`
  e **não grava** a sessão.

## Fluxo e orçamento

1. Lê a origem e obtém o **lock** entre processos (`<url-controle>.lock`,
   `flock` exclusivo da `std`; espera até 120 s, depois
   `LoginEmAndamentoEmOutroProcesso`).
2. Sem `--forcar`, confere a sessão gravada com **um GET** da URL do
   Controle (mesmas travas do GET da caixa). Controle reconhecido →
   `sessao_existente_valida`, sem login (orçamento: 1). Tela de login,
   3xx ou 401/403 → segue para o login. Outra falha (5xx, rede, MIME,
   Controle irreconhecível) sai tipada, sem login. Arquivos ausentes ou
   vazios (os que o `ies setup` exige antes do primeiro login), ou de
   outra geração (abaixo), vão ao login sem esse GET.
3. GET da raiz `https://host/` e redirecionamentos até a tela de login.
4. Um POST derivado do formulário servido.
5. Redirecionamentos até `acao=procedimento_controlar` com `infra_hash`
   da instalação; tela de Painel de Controle: um GET do link do menu
   `lnkControleProcessos` que a tela publica.

| Família observada | Antes do POST | POST | Depois do POST |
| --- | --- | --- | --- |
| SIP direto (UnB) | GET raiz → 302, GET `sip/login.php` (2) | 1 | redirecionamentos (passa por `inicializar.php`) até o Controle, + 1 se Painel |
| SIP via `/sei/` (PEN1, UFRJ) | GET raiz, GET `/sei/`, GET `sip/login.php` (3) | 1 | idem |
| Módulo próprio (GO) | GET raiz, GET `/sei/`, GET `MdGoiasLoginSEI.php` em HTTPS (3) | 1 | idem |

Cada cadeia tem no máximo **8** redirecionamentos. `requisicoes` na
saída conta todos os pedidos feitos (inclusive o GET da conferência).

**Efeito local: índice.** O Controle já obtido (o do passo 2, quando a
sessão gravada vale, ou o do fim do passo 5, depois de gravar a sessão)
passa pela
[colheita do Controle](../../docs/arquitetura/0003-indice-local.md):
NUP ↔ id e o link `procedimento_trabalhar` de cada linha válida, o link
da pesquisa e o `action` da pesquisa rápida, com a geração do cookie
gravado e a unidade da URL gravada. Só arquivo local (`ies.sqlite3`, ou
nada com `IES_SEM_INDICE`); nenhuma requisição a mais, e a saída e as
falhas do login não mudam (falha do banco nunca derruba o comando).

## Redirecionamentos do login (regra observada)

Só o fluxo de login segue redirects, por um interpretador explícito; os
GET/POST dos outros comandos continuam recusando 3xx. `Location` é
resolvido contra a URL atual; só vale no host da instalação. `http://`
no **mesmo host** é promovido a `https://` (UFRJ e GO publicam
`http://`; a URL HTTP nunca é pedida, e nenhum cookie ou credencial
trafega sem TLS). Outro host, porta, usuário na URL, esquema
diferente, segmentos `.`/`..`, `Location` ausente, mais de 8 saltos e
307/308 depois do POST (repetiriam as credenciais) são recusas tipadas.

## Cookies

Pote limitado ao host da instalação: guarda nome, valor e `Path`
(padrão RFC 6265 quando ausente), aceita `Domain` só se ele contém o
host, aplica `Max-Age<=0` e `Expires` de ano passado como remoção. Todo
pedido é HTTPS; `Secure` não restringe nada. Valor fora do conjunto
aceito pelo arquivo de cookie é recusa. Valores nunca aparecem em
`Debug`, logs ou saída. O arquivo de cookie recebe `nome=valor; …` dos
cookies que o navegador enviaria ao `controlador.php` da instalação.

## Formulário

Derivado pelo primitivo de formulário (`src/formulario_web`), com
`const` por operação em `src/sessao_login/tela.rs`: `frmLogin` único,
`POST` urlencoded, `action` relativo à **URL servida** (no GO, à URL do
módulo) e no mesmo host; ocultos como servidos; `txtUsuario`,
`pwdSenha` das credenciais; `selOrgao` = a única opção (fora `null`)
cujo texto visível ou `value` é **igual** ao órgão configurado;
`hdnAcao` = o inteiro literal de `onsubmit="return acaoLogin(N);"`,
desde que o script servido grave `hdnAcao` com o argumento. Botão:
`Acessar` (UnB) participa com o valor servido; `sbmAcessar`
(PEN1, UFRJ, GO) é desabilitado pelo `acaoLogin(2)` servido antes de o
navegador montar o corpo, e não participa. Combinação não observada é
recusa. Corpo no charset da tela (`iso-8859-1` observado), `Origin`
do host e `Referer` = URL da tela.

## Proxy do ambiente

O login usa o cliente HTTPS único do IES (`ureq` com `native-tls`, todos
os cabeçalhos guardados, inclusive cada `Set-Cookie`), com as regras de
proxy do [transporte HTTPS](transporte-https.md#transporte-https-um-cliente-proxy-do-ambiente-em-todos-os-comandos):
o proxy do ambiente é lido pelo `ureq`, a credencial do proxy vai só no
`CONNECT` e nunca em `Debug`, log ou saída. Proxy inválido ou túnel
recusado é falha de transporte do proxy, sem `status_http` (o SEI não
respondeu) e sem retry.

## Saída

```json
{"instalacao":"https://sei.exemplo.gov.br/sei","unidade_exibida":"SIGLA",
 "conexao":"login_realizado","requisicoes":6}
```

`conexao` é `login_realizado` ou `sessao_existente_valida`. Nunca sai
credencial, cookie, URL assinada ou HTML.

## Gravação, geração e lock

Cookie e URL são gravados por arquivo temporário `0600` no mesmo
diretório, `fsync` e `rename`. O marcador `<url-controle>.geracao`
(`0600`, JSON com `geracao`, `instalacao` e o SHA-256 do cookie e da URL)
é renomeado **por último**: arquivos que não batem com o marcador são de
gerações misturadas e não são conferidos (vão direto ao login). Sem
marcador (sessão preparada à mão), vale a conferência por GET. Marcador
existente e ilegível falha a gravação (`MarcadorDaSessaoIlegivel`): a
geração nunca recomeça em 1 por ilegibilidade. O lock
cobre conferência, login e gravação; o segundo processo espera e confere
a sessão que o primeiro gravou.

Os outros comandos leem a sessão sem o lock, num retrato coerente (#222):
o marcador antes e depois de URL, origem e cookie. Marcador que mudou, ou
que não confere com URL e cookie lidos, faz reler os arquivos locais uma
vez (nenhuma requisição); de novo incoerente, a leitura falha com
`SessaoTrocadaDuranteALeitura` (`FalhaTransporte`, saída 1, sem relogin;
rodar de novo resolve). Todas as requisições do comando e a colheita do
índice usam o cookie desse retrato. Sem marcador, a leitura é a de antes
([decisão](../../docs/decisoes/sessao-login/leitura-coerente-da-sessao.md)).

## Falhas tipadas

`caminho_credenciais_nao_informado`, `CampoDasCredenciaisAusente`,
`CampoDasCredenciaisRepetido`, `CampoDasCredenciaisVazio`,
`LinhaDasCredenciaisSemCampoConhecido`, os de arquivo privado
(`ArquivoDeCredenciais`, `OrigemIlegivel`, `ArquivoDaSessao`, cada um com
o `ErroArquivoPrivado` do arquivo), `OrigemNaoAutorizada`,
`LoginEmAndamentoEmOutroProcesso`,
`ArquivoDeLockDoLoginInacessivel`, `controle_web_nao_reconhecido`
(sessão existente), `redirecionamento_do_login_sem_localizacao`,
`LocalizacaoDeRedirecionamentoNaoInterpretavel`,
`RedirecionamentoParaForaDaInstalacao`,
`RedirecionamentosAcimaDoLimite`,
`RedirecionamentoQueRepetiriaOEnvioDoLogin`,
`CookieRecebidoForaDoFormatoAceito`, `tela_de_login_nao_reconhecida`,
`CharsetDaTelaDeLoginNaoSuportado`, `captcha_exigido`,
`SegundoFatorExigido`, `selecao_de_orgao_ausente_na_tela_de_login`,
`selecao_de_orgao_repetida_na_tela_de_login`,
`orgao_nao_encontrado_na_tela_de_login`,
`orgao_com_mais_de_uma_opcao_na_tela_de_login`,
`OnsubmitDoLoginAusente` (o `<form>` sem `onsubmit`),
`OnsubmitDoLoginForaDoFormato` (`onsubmit` diferente de `return acaoLogin(N);`),
`ScriptDoLoginSemHdnAcao` (a página não grava `hdnAcao`),
`botao_de_envio_do_login_nao_reconhecido`, os do primitivo de formulário,
`credencial_recusada` (com `mensagem_do_servidor` saneada quando a tela
a declara: primeiro o `#txaInfraValidacao` da
[validação da Infra](../../docs/como-o-sei-funciona/validacao-da-infra.md),
observado na UnB 4.0.9; senão `divInfraExcecao`, `alert-danger`,
`infraMensagemErro` ou `infraExcecao`), `tela_apos_o_login_nao_reconhecida`,
`link_do_controle_de_processos_nao_reconhecido`,
`ControleAposOLoginNaoReconhecido`, `CookieDaSessaoNaoRecebido`,
`DestinoDaSessaoNaoEhArquivoRegular` (o nome final é link ou não
é arquivo regular), `DiretorioDaSessaoGravavelPorOutros` (o
diretório real da sessão é gravável por grupo ou outros sem sticky bit;
link no caminho, como `/home` → `/var/home`, é aceito),
`GravacaoDaSessaoFalhou`,
`UnidadeExibidaDiferenteDaSolicitada` e os de transporte
(`Status(StatusHttpRecusado { status, mensagem_do_servidor })`,
`Rede`, `TipoDeConteudoNaoHtml`, `RespostaExcessiva`) e
os do proxy (acima).
Nenhuma recusa repete o POST: captcha, 2FA ou credencial recusada
terminam o comando. A falha sai com 1 e `Error: <ErroLogin::…>`
em stderr: `Transporte(Status(StatusHttpRecusado { status, mensagem_do_servidor }))`
nas recusas HTTP, `CredencialRecusada { mensagem_do_servidor }` e, nas
falhas do arquivo de credenciais, o campo (a chave, nunca o valor), como
em `CampoDasCredenciaisAusente(Senha)`.

## Relogin único nas leituras

Refs #200; [decisão](../../docs/decisoes/sessao-login/relogin-de-leitura.md).
Vale para `ies processo NUP`, `processo buscar`, `documento DOC`,
`documento buscar`, `unidade listar-processos`, `recebidos`, `gerados`,
`atual` e `colunas` sem `--definir`. **Escrita nunca reloga**:
`concluir`, `reabrir`, `enviar`, `assinar` e `colunas --definir`, com ou
sem `--confirmar` (a prévia também não, para não misturar sessões entre a
prévia e a escrita), saem com o erro tipado e o agente decide.

1. No início do comando, a geração gravada: os SHA-256 do cookie e da URL
   do Controle (os do marcador), lidos dos arquivos, sem rede.
2. A leitura roda. Só `PaginaDeLoginRecebidaNoControle`,
   `ControleRedirecionadoParaOLogin` e o marcador `SessaoEncerrada`
   disparam o relogin (decisão por `match` em cada tipo de erro:
   `sessao_encerrada()`). `ControleRedirecionadoParaOLogin` é o **302 do GET do Controle** cujo `Location`, resolvido
   contra a instalação (mesmo host; `http://` no mesmo host conta), é uma
   entrada do login dela: `/sip/login.php`, o módulo de Goiás
   (`<base>/modulos/goias/login-sei/MdGoiasLoginSEI.php` ou `login.php`)
   ou a raiz `<base>/`, sem fragmento e sem `msg=Link sem assinatura`;
   carrega o status 302 e nunca é seguido. Nenhuma outra
   falha dispara: `Status` 401/403, `BloqueioDeSegurancaDaInstalacao`
   (WAF), `Status` 3xx (qualquer outro 3xx, outro host ou caminho, 302
   sem `Location`), os demais `Status`, `ErroProcessoObter::EntradaPorIdRecusada`,
   `TelaDeLoginRecebidaAposEnvio`, rede, proxy e tela inesperada.
3. Sem `IES_CREDENCIAIS_FILE`, ou com o arquivo ausente: comportamento
   anterior, sem lock e sem pedido a mais. Arquivo presente e inválido
   (permissão, link, formato) sai com a falha tipada dele.
4. O lock do login (até 120 s). Sob o lock, a geração gravada é
   comparada com a do passo 1. **Mudou**: outro processo já relogou; a
   leitura usa a sessão nova e nada é pedido. **Igual**: login pelos
   passos 3 a 5 do fluxo acima (sem o GET de conferência, porque a
   leitura acabou de ver a sessão vencida), `--unidade` conferida antes
   de gravar, gravação da nova geração e colheita do Controle obtido.
5. A leitura recomeça **do início, uma vez**, relendo sessão e índice.
   Se falhar de novo (qualquer falha), sai com o erro, sem segundo login.

Orçamento: N₁ (a primeira tentativa, até a falha) + o login da família
(tabela acima; UnB: 5) + N₂ (a leitura inteira). Geração trocada:
N₁ + N₂. Sem retry de outra natureza.

**Saída.** Sem recomeço, nada muda. Com recomeço:

- `processo NUP`, `processo buscar`, `documento DOC` e `documento buscar`
  já publicam `requisicoes`: o valor passa a ser N₁ + login + N₂ (também
  no texto humano).
- `unidade atual`, `colunas`, `listar-processos` (em cada lista do array),
  `recebidos` e `gerados`, e a linha de `--saida`, ganham `requisicoes`
  com esse total. O arquivo de `--saida` não muda.
- `relogin: true` quando **este** processo logou; ausente quando a
  geração já tinha trocado.
- Falha depois do relogin: o erro da leitura com o contexto
  `LeituraRefeitaAposRelogin(NotaDoRelogin)`. Falha do relogin
  (`CredencialRecusada`, `LoginEmAndamentoEmOutroProcesso`,
  `UnidadeExibidaDiferenteDaSolicitada`...): o `ErroLogin` do
  `ies login`, sem reler. Saída 1.
- Sem `--json`, uma linha em `stderr` diz que a leitura recomeçou, com o
  total de requisições.

A contagem soma todos os pedidos do processo pelo cliente HTTPS único
(registrados antes do envio). Credenciais, cookie e URL nunca saem.

## Fora do escopo

Relogin em escrita, prévia de escrita, `ies status` e `ies setup`;
relogin em laço ou por outra falha. Captcha e 2FA não são resolvidos.
REST e SOAP não são usados.
[Decisão](../../docs/decisoes/sessao-login.md),
[evidência](../evidencias/sessao-login.md),
[observações](../../docs/como-o-sei-funciona/sessao-login.md) e
[roteiro live](../../docs/validacao-live/roteiro-login.md).
