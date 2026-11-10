---
type: Contract
title: Recusas HTTP e mensagem do SEI
description: Variante tipada por fato nas recusas de resposta do GET, status HTTP e mensagem saneada do SEI nas recusas, parte do contrato da CLI
---

# Recusas HTTP e mensagem do SEI

Parte do [contrato da CLI](../cli.md), separada por tamanho.

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
([validação da Infra](../../docs/como-o-sei-funciona/validacao-da-infra.md)),
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
{"codigo":"fonte_invalida", "erro":"o SEI respondeu com erro do servidor (5xx)","retentavel":false,
 "status_http":500,"mensagem_do_servidor":"Erro acessando o banco de dados."}
```

401/403 continuam `sessao_nao_autorizada`, agora com `status_http` e a
mensagem; 3xx continua `redirecionamento_recusado`, com `status_http`,
salvo o 302 do GET do Controle para a entrada do login da própria
instalação, que é `sessao_encerrada_rode_ies_login`, também com
`status_http` 302 ([relogin](sessao-login.md#relogin-único-nas-leituras)).
O 403 com a página de bloqueio de segurança da instalação (`<title>Acesso
Negado</title>` e "bloqueada por nossa política de segurança" no texto
visível; visto em GO), ou a do Cloudflare (`<title>Attention Required! |
Cloudflare</title>` e "Sorry, you have been blocked"; visto na PEN2 e na PEN3), é
`bloqueio_de_seguranca_da_instalacao`, com
`status_http` 403 e a mensagem saneada, que traz o código de bloqueio. A
sessão não venceu: `ies login` não reloga por ele, e nada é repetido.
