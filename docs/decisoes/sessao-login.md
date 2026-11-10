---
type: ImplementationDecision
title: Login Web nativo do IES pela tela servida, com sessão gravada por geração
operation: sessao.login
implementation: partial
status: adopted-for-scope
scope: ies login [--forcar] na instalação configurada; credenciais em arquivo privado 0600; um GET de conferência da sessão gravada; login pela raiz com redirecionamentos só no host; um POST derivado do frmLogin servido; gravação atômica de cookie, URL e marcador de geração sob lock; nos outros comandos, só o relogin único das leituras do apêndice relogin-de-leitura (nunca na escrita)
selected_strategy: ies-login-web-nativo
alternatives: ies-login-web-nativo,todos-como-harness-externo,rest-wssei-token,soap-official
evidence_level: comportamento-do-todos-telas-publicas-de-login-e-ensaios-live
live_validation: pass-unb-pen1-pen3-pen4-ufrj-go-pen2-bloqueada-pelo-ambiente
---

# Login Web nativo do IES

## Alternativas consideradas

- **todos-como-harness-externo (situação até aqui):** o operador loga com
  o Todos (ou o navegador) e copia cookie e URL assinada do Controle para os
  arquivos `0600` que o IES lê. Zero código de login no IES.
- **ies-login-web-nativo (escolhida):** `ies login` interpreta a tela de
  login servida pela raiz da instalação, envia um POST derivado pelo
  primitivo de formulário e grava os mesmos arquivos.
- **rest-wssei-token:** autenticar pelo módulo REST opcional e usar o
  token. O módulo falta em muitas instalações, e o token não autoriza a
  interface Web que os comandos usam.
- **soap-official:** o SOAP do SEI tem credencial própria de sistema, não
  a do usuário Web; não produz a sessão Web.

## Comparação

O harness externo exige uma segunda ferramenta, copiar capacidade assinada
à mão e não controla geração nem concorrência: cookie de uma sessão e URL
de outra derrubam a sessão. O login nativo põe o comportamento observado
(redirecionamentos, órgão, `hdnAcao`, botão, Painel) em código testado,
com orçamento medido e falhas tipadas, ao custo de manter um interpretador
de redirecionamentos e um cliente HTTP próprios. REST e SOAP respondem
outra pergunta (outra credencial, outro escopo) e ficam fora, como manda a
prioridade de interfaces (Web principal; REST baixa; SOAP não prioritário).

## Decisão

- **Entrada:** `IES_CREDENCIAIS_FILE` (`usuario=`, `senha=`, `orgao=`),
  lido só quando o login é necessário; nunca argumento.
- **Redirecionamentos:** interpretador explícito só do login, até 8 saltos
  por cadeia, `Location` resolvido contra a URL atual, só o host da
  instalação. Regra observada: `http://` no mesmo host vira `https://`
  (UFRJ na raiz, GO no módulo), sem pedir a URL HTTP. 307/308 depois do
  POST é recusa (repetiria a senha). Os outros comandos seguem sem redirect.
- **Órgão:** igualdade exata com o texto visível (entidades decodificadas,
  espaços colapsados) ou o `value` da opção, fora `null`; nenhuma ou mais
  de uma opção são recusas distintas (decisão do proprietário). O Todos
  usa antes a opção `selected` e, sem casamento exato, aceita a sigla
  como palavra do texto, sem caixa; aqui nada disso vale.
- **`hdnAcao` e botão:** `onsubmit="return acaoLogin(N);"` com o trecho
  servido que grava `hdnAcao`; `Acessar` participa com o valor servido;
  `sbmAcessar` é desabilitado pelo `acaoLogin(2)` servido e, como no
  navegador, não participa (o Todos o enviava com o texto do botão).
  Caso novo do primitivo: `EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit`.
- **Primitivo:** `derivar_submissao_de_login` confere a tela e o `action`
  no host (o SIP fica fora do caminho-base do `controlador.php`); `const`
  por operação em `src/sessao_login/tela.rs`, sem despachante.
- **Cookies:** pote por host e `Path`; o arquivo recebe os que o navegador
  enviaria ao `controlador.php`.
- **Cliente HTTP:** HTTP/1.1 mínimo sobre `native-tls`; o `minreq` 2.14.1
  guardava um valor por cabeçalho e perderia os `Set-Cookie` múltiplos
  observados (UnB, PEN1, GO). Mesmo limite de 8 MB e tempo de 30 s,
  `User-Agent: ies/<versão>`. Depois virou o cliente HTTPS único de todos
  os comandos (`src/transporte/cliente_https.rs`, com proxy do ambiente) e
  o `minreq` saiu das dependências.
- **Geração e lock:** marcador `<url-controle>.geracao` (contador e SHA-256
  de cookie e URL) renomeado por último; lock `<url-controle>.lock` com
  `File::try_lock` da `std` (sem dependência nova). Arquivos que não batem
  com o marcador são de gerações misturadas e não são conferidos.
- **Sem login desnecessário:** um GET da sessão gravada antes; `--forcar`
  pula. Falha que não é de sessão vencida não vira login.
- **Sucesso:** só com a URL assinada de `procedimento_controlar` da
  instalação e o Controle reconhecido pela leitura da caixa.

Descartado: guardar a "receita" do POST para logar sem a tela (o Todos o
faz; uma tela diferente da guardada já foi confundida com recusa) e
relogar em qualquer falha nos outros comandos. O relogin único das
leituras, só depois da falha tipada de sessão encerrada, conferindo a
geração sob o lock e nunca na escrita, está no apêndice
[relogin de leitura](sessao-login/relogin-de-leitura.md) (Refs #200).

## Evidências verificáveis

- [Contrato](../../contracts/cli/sessao-login.md) e
  [card de evidência](../../contracts/evidencias/sessao-login.md).
- [Observações do login](../como-o-sei-funciona/sessao-login.md).
- [Testes da CLI sem rede](../../tests/sessao_login.rs) e os testes do
  binário em `src/sessao_login/tests_*.rs` (telas sintéticas por família e
  sequência exata de pedidos).
- [Roteiro live do operador](../validacao-live/roteiro-login.md).
- Ensaios live com `ies login`: PASS em UnB e PEN1
  ([colheita, 03/11/2026](../validacao-live/2026-11-03-colheita-do-controle.md)),
  GO ([árvore, 03/11/2026](../validacao-live/2026-11-03-processo-obter-arvore.md)),
  PEN3, PEN4 e UFRJ ([paginação, 04/11/2026](../validacao-live/2026-11-04-unidade-paginacao.md));
  PEN2 recusada pelo ambiente (503 e 403 do Cloudflare,
  [2ª tentativa](../validacao-live/2026-11-04-unidade-paginacao-go-pen2.md)).

## Limitações e revisão

Nenhum login autenticado foi feito pelo IES antes do roteiro: o contêiner
do agente não tem rede para o SEI. A mensagem de recusa do SIP não foi
capturada; a saída a publica só se a tela devolvida a declarar nos
contêineres que o Todos lê. Captcha e 2FA não são resolvidos. Revisar se o
roteiro falhar em alguma família (por exemplo, se o SIP exigir o botão
`sbmAcessar` no corpo), se surgir outra forma de `onsubmit`, botão,
`Location` ou link do Painel, ou se uma instalação servir o login fora do
host da instalação.
