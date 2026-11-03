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
  Diferença literal falha com `unidade_exibida_diferente_da_solicitada`
  e **não grava** a sessão.

## Fluxo e orçamento

1. Lê a origem e obtém o **lock** entre processos (`<url-controle>.lock`,
   `flock` exclusivo da `std`; espera até 120 s, depois
   `login_em_andamento_em_outro_processo`).
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

O cliente HTTPS único do IES (`native-tls`, todos os cabeçalhos
guardados, inclusive cada `Set-Cookie`) atravessa um proxy declarado no
ambiente. O login e **todos os outros comandos** usam o mesmo cliente e
as mesmas regras ([transporte HTTPS](../cli.md#transporte-https-um-cliente-proxy-do-ambiente-em-todos-os-comandos)).

- Variável: a primeira **não vazia** de `https_proxy`, `HTTPS_PROXY`,
  `all_proxy`, `ALL_PROXY` (o destino é sempre HTTPS; `http_proxy` não
  vale). O IES segue a convenção do curl, documentada aqui.
- Exceções: `no_proxy`, senão `NO_PROXY`, separadas por vírgula, sem
  caixa: `*` dispensa todos; `.sufixo` dispensa o domínio e os
  subdomínios; outra entrada é host exato. Entrada com porta ou
  curinga parcial (`*.x`) não casa. A exceção é conferida antes do
  endereço: host dispensado conecta direto mesmo com proxy inválido.
- Endereço `[http://][usuario[:senha]@]host:porta[/]` (IPv6 entre
  colchetes); porta obrigatória (sem padrão adivinhado); `%XX` da
  credencial decodificado. Credencial vai só em
  `Proxy-Authorization: Basic`, nunca em `Debug`, log ou saída.
  `https://`, `socks5://` e outros esquemas: `proxy_com_esquema_nao_suportado`.
- Por pedido: conexão TCP ao proxy (cada endereço resolvido), um
  `CONNECT host:443 HTTP/1.1` com `Host`, resposta lida byte a byte até a
  linha vazia (16 KiB no máximo); 2xx abre o túnel e o TLS segue com o
  host do SEI (SNI e certificado do SEI pela CA do sistema, como sem
  proxy). Mesmos 30 s de tempo limite e limites de resposta. Sem retry,
  nem do `CONNECT` nem do pedido.

| Fato | Subcódigo |
| --- | --- |
| Valor fora do formato acima | `endereco_de_proxy_invalido` |
| Esquema diferente de `http://` | `proxy_com_esquema_nao_suportado` |
| DNS ou TCP do proxy falhou (nada foi ao SEI) | `proxy_inacessivel` |
| Resposta ao `CONNECT` fora de HTTP/1.x ou truncada | `resposta_do_proxy_nao_interpretavel` |
| `CONNECT` respondido fora de 2xx (`407`, `403`, `502`...) | `proxy_recusou_o_tunel`, com `status_do_proxy` |

`status_do_proxy` não é `status_http` (o SEI não respondeu). Testado
offline com proxy falso local (`TcpListener`); o TLS através do túnel
não tem teste automatizado.

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
marcador (sessão preparada à mão), vale a conferência por GET. O lock
cobre conferência, login e gravação; o segundo processo espera e confere
a sessão que o primeiro gravou.

## Falhas tipadas

`caminho_credenciais_nao_informado`, `campo_das_credenciais_ausente`,
`campo_das_credenciais_repetido`, `campo_das_credenciais_vazio`,
`linha_das_credenciais_sem_campo_conhecido`, os de arquivo privado
(`arquivo_privado_*`), `login_em_andamento_em_outro_processo`,
`arquivo_de_lock_do_login_inacessivel`, `controle_web_nao_reconhecido`
(sessão existente), `redirecionamento_sem_localizacao`,
`localizacao_de_redirecionamento_nao_interpretavel`,
`redirecionamento_para_fora_da_instalacao`,
`redirecionamentos_acima_do_limite`,
`redirecionamento_que_repetiria_o_envio_do_login`,
`cookie_recebido_fora_do_formato_aceito`, `tela_de_login_nao_reconhecida`,
`charset_da_tela_de_login_nao_suportado`, `captcha_exigido`,
`segundo_fator_exigido`, `selecao_de_orgao_ausente_na_tela_de_login`,
`selecao_de_orgao_repetida_na_tela_de_login`,
`orgao_nao_encontrado_na_tela_de_login`,
`orgao_com_mais_de_uma_opcao_na_tela_de_login`,
`regra_do_onsubmit_do_login_nao_observada`,
`botao_de_envio_do_login_nao_reconhecido`, os do primitivo de formulário,
`credencial_recusada` (com `mensagem_do_servidor` saneada quando a tela
a declara: primeiro o `#txaInfraValidacao` da
[validação da Infra](../../docs/como-o-sei-funciona/validacao-da-infra.md),
observado na UnB 4.0.9; senão `divInfraExcecao`, `alert-danger`,
`infraMensagemErro` ou `infraExcecao`), `tela_apos_o_login_nao_reconhecida`,
`link_do_controle_de_processos_nao_reconhecido`,
`controle_apos_o_login_nao_reconhecido`, `cookie_da_sessao_nao_recebido`,
`destino_da_sessao_nao_eh_arquivo_regular`, `gravacao_da_sessao_falhou`,
`unidade_exibida_diferente_da_solicitada` e os de transporte
(`status_http_*` com `status_http` e `mensagem_do_servidor`,
`falha_de_rede`, `tipo_de_conteudo_nao_html`, `resposta_excessiva`) e
os do proxy (acima).
Nenhuma recusa repete o POST: captcha, 2FA ou credencial recusada
terminam o comando. O JSON é o `fonte_invalida` dos outros comandos, com
`status_http` e `mensagem_do_servidor` nas recusas HTTP, só
`mensagem_do_servidor` em `credencial_recusada`, `status_do_proxy` em
`proxy_recusou_o_tunel` e `campo` (a chave, nunca o valor) nas falhas do
arquivo de credenciais. Saída 1.

## Fora do escopo

Outros comandos **não** fazem login automático nesta fase. Futuro:
relogar só depois de uma falha tipada de sessão vencida, uma vez, nunca
em laço. Captcha e 2FA não são resolvidos. REST e SOAP não são usados.
[Decisão](../../docs/decisoes/sessao-login.md),
[evidência](../evidencias/sessao-login.md),
[observações](../../docs/como-o-sei-funciona/sessao-login.md) e
[roteiro live](../../docs/validacao-live/roteiro-login.md).
