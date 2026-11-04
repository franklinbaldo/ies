---
type: ArchitectureDecision
title: Pré-condições por família de GET e POST HTTPS
description: Inventário documental de origem, cookie, redirects e limites antes de qualquer consolidação do transporte
status: proposed
decision: document preconditions first; preserve distinct redirect and binary response semantics
---

# 0004 — Pré-condições HTTP no cliente Web

## Escopo

Inventário da `main@7a8b332`, antes de qualquer refatoração. Os caminhos
abaixo incluem os pedidos iniciados pelos comandos atuais e os adaptadores
que os enviam. Replays de cassette não fazem rede. Os fluxos ainda abertos
de `processo enviar` e `documento assinar` precisam ser conferidos depois que
as alterações do Claude Code entrarem na base.

O transporte comum em [`cliente_https.rs`](../../src/transporte/cliente_https.rs)
abre uma conexão por pedido, usa HTTPS, timeout de 30 s, não segue redirects
e não repete pedidos. Ele valida a forma da URL e o host, mas não conhece a
instalação autorizada. Essa conferência ocorre no chamador ou na derivação do
formulário e, portanto, não é idêntica em todas as rotas.

## Inventário

| Pedido e chamadores | Origem da URL e conferência da instalação | Cookie | 3xx | Limite de resposta |
| --- | --- | --- | --- | --- |
| GET da tela de controle: `ler_tela_do_controle` (`status_web`, `unidade atual`, listas e colunas) | URL e origem vêm dos arquivos privados da sessão; `ler_tela_via` confere HTTPS, host e caminho-base antes de abrir o cookie. | Cookie privado validado contra a mesma origem antes da chamada. | Recusa tipada; corpo do 3xx não é lido. | HTML: 8 MB durante a leitura. |
| GET HTML publicado: `EnvioDoControle::get` (`pesquisa_web/fluxo`, `visao_do_controle/nivel`) | Link vem de tela já servida; o leitor do link confere a instalação e a rota antes do GET. A camada genérica `get_https` só confere URL/host sintáticos. | Cookie privado validado antes do pedido. | Recusa tipada; não segue. | HTML: 8 MB durante a leitura; exige MIME HTML e status 200. |
| GET de processo com `Location`: `EnvioDoProcesso::get_sem_seguir` e `Rede::get` (`processo_obter`) | Link assinado guardado é validado ao entrar no índice e chaveado por instalação, geração e unidade; links de trabalho derivados de 302 são conferidos pelo fluxo. A entrada por id usa só as formas sem assinatura da lista branca. O adaptador HTTP não repete essa autorização. | A sessão é validada antes de criar `Rede`; o mesmo cookie é enviado nos GETs subsequentes. | 302 volta com `Location` como dado, sem ler corpo; o chamador decide se o destino serve e o confere. Outros status são classificados. | Resposta HTML: 8 MB durante a leitura. |
| POST de formulário observado: `enviar_submissao_via` (pesquisas, paginação, troca de visão/unidade, histórico e formulários de processo) | `SubmissaoDerivada` vem do formulário servido e resolve `action` contra a URL dessa tela; a derivação confere a instalação, a ação e, conforme operação, a unidade. `enviar_post` envia essa URL, `Origin` e `Referer`. | Arquivo privado lido e validado imediatamente antes do transporte. | O helper devolve 302 como dado, sem ler corpo nem seguir. Cada operação classifica a resposta (por exemplo, resultado único, paginação recusada ou próximo passo validado). | HTML de resposta: 8 MB durante a leitura. O POST envia `Content-Length` exato do formulário; não foi encontrado limite separado para o corpo enviado. |
| GET/POST do login: `sessao_login/fluxo` e `sessao_login/http` | GET inicial vem da raiz da `Instalacao`; próximos GETs resolvem apenas `Location` no mesmo host, até 8 saltos. POST é derivado do formulário de login servido. | Pote de cookies da sessão, escopado por URL; absorve `Set-Cookie` do servidor. | O transporte não segue. O fluxo interpreta manualmente os 3xx, valida o destino e continua só dentro do host permitido. | Corpos de GET e POST limitados a 8 MB. O login lê e processa o corpo do redirecionamento. |
| GET do Nível de detalhe: `obter_tela_do_nivel` | URL extraída do `onclick` servido, resolvida contra a tela e conferida com `Instalacao::autoriza_controlador`. | Cookie privado da sessão revalidado antes de cada GET. | Recusa tipada; não segue. | HTML: 8 MB durante a leitura; status 200 e MIME HTML. |
| GET binário do anexo: `processo_obter::Rede::get_binario` → `resposta_binaria::get_binario` | Link assinado vem do nó da árvore validada. A autorização ocorre no leitor da árvore; o cliente HTTPS só valida a forma da URL. | Cookie da sessão, já validado pelo fluxo. | Não segue; 3xx é recusado pela leitura do anexo. | Limite específico de anexo, configurado pelo chamador (padrão 64 MiB; texto limitado também a 8 MB). Corpo em fluxo. |

Chamadores de `enviar_submissao_via` também incluem as escritas atuais de
`processo concluir` e `processo reabrir`; a operação acrescenta confirmação,
classificação do efeito e eventual verificação posterior. A mesma primitiva
de POST não torna equivalentes os seus contratos. Não incluí testes de
integração ao vivo.

## O que pode ser consolidado

- **Candidato:** reunir a leitura de resposta HTML (status, MIME, charset e
  limite em fluxo) usada pelos GETs textuais. As rotas de controle, tela
  publicada e `Rede::get` compartilham o teto de 8 MB e a recusa de redirects.
  A extração só é segura se mantiver as verificações de origem no ponto em que
  cada URL é construída e mantiver 302 tipado como dado onde o chamador precisa
  do `Location`.
- **Manter separado:** a cadeia de redirects do login, os GETs que devolvem
  `Location` ao leitor de processo, e o GET binário. Eles têm regras distintas
  de cookie, destino e corpo.
- **Não generalizar a origem na camada HTTP:** `cliente_https` não tem contexto
  de instalação. A tela do Controle, a derivação de formulários, os links
  assinados, os destinos de 302 e as duas formas permitidas de entrada por id
  exigem validações diferentes. Uma validação única aplicada ao transporte
  poderia aceitar uma origem imprópria ou rejeitar uma rota medida.

## Verificação documental

Fontes conferidas: `transporte/cliente_https.rs`, `transporte/envio_post.rs`,
`transporte/resposta_http.rs`, `formulario_web.rs`, `sessao_login/{fluxo,http}.rs`,
`pesquisa_web/fluxo.rs`, `processo_obter/{abertura,fluxo,pastas,andamentos,propriedades,marcadores,documento_indice,documento_versao}.rs`,
`visao_do_controle/{envio,nivel}.rs` e `pagina_do_controle/envio.rs`. O GET de
anexo está descrito a partir do contrato existente; seu código não foi alterado.
