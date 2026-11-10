---
type: ArchitectureDecision
title: Pré-condições por família de GET e POST HTTPS
description: Inventário documental de origem, cookie, redirects e limites antes de qualquer consolidação do transporte
status: proposed
decision: document preconditions first; preserve distinct redirect and binary response semantics
---

# 0004 — Pré-condições HTTP no cliente Web

## Escopo

Inventário original de `main@7a8b332`, atualizado para os fluxos de escrita
integrados até `main@8a25a5e` e para o handoff #237 (base #233). A tabela
inicial preserva o retrato histórico; a matriz complementar abaixo distingue
as pré-condições atuais, sem afirmar validação live ou cobertura integral.

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

## Complemento: escritas integradas e fronteiras (#237)

Os fluxos abaixo são distintos, embora compartilhem partes do transporte.
A URL assinada continua sendo tratada como credencial de sessão; nunca
se reconstrói seu valor nem se faz retry de operação potencialmente efetiva.

| Fluxo | Leitura e proveniência | Envio e resposta | Efeito e verificação |
| --- | --- | --- | --- |
| Concluir | `processo_concluir/fluxo.rs`: abre o processo e confere a barra; na modalidade por tela, lê HTML, MIME, charset e deriva o formulário observado. | `enviar_submissao_via` emite um POST; o 302 é dado. A modalidade por função usa GET do link conferido, sem seguir redirect. | Após confirmação, relê a situação na árvore; falha após o envio pode resultar em escrita não confirmada. |
| Reabrir | `processo_reabrir/fluxo.rs`: abre o processo, lê a tela do nó e extrai a função servida. | GET pela função validada, nunca acompanhado de retry; 3xx não é seguido. | Relê a árvore e reconcilia a situação da unidade. |
| Enviar | `processo_enviar/fluxo.rs`: abre o processo, lê a tela de envio, consulta o autocomplete por POST derivado (consulta, não envio de processo) e resolve a unidade. | O POST efetivo deriva campos, charset e action da tela; 302 retorna como dado. | Relê a árvore e concilia o envio, mesmo se houve falha após a tentativa. |
| Assinar | `documento_assinar/fluxo.rs`: obtém a tela e deriva o formulário com cargo/assinante. | Envia POST derivado sem repetir. A resposta esperada é HTML com confirmação; 302 é recusado pelo fluxo. | Reabre o documento na árvore e compara assinaturas. Senha e cookies não podem aparecer em logs ou fixtures. |
| Trocar unidade | `unidade_trocar.rs`: lê o Controle e a tela de troca; extrai a unidade e a action servidas. | Um POST derivado, seguido apenas dos saltos 302 explicitamente verificados; cada GET valida o destino na operação. | Confere o Controle da unidade nova; desvios após o POST sãa escrita não confirmada. |

### Matriz sintética e resultado observável

| Caso | Bancada | Garantia de teste |
| --- | --- | --- |
| Outro host, porta, userinfo e referência `//host` | `transporte/web_mock_tests.rs::matriz_url_host_porta_userinfo_hash_e_duplicatas_sem_cookie` | Rejeição antes de ler cookie ou chamar o transporte. |
| `infra_hash` inválido/repetido e `acao` repetida | Mesma bancada | Rejeição de URL antes de GET; duplicatas genéricas ficam em teste ignorado, pendentes na #241. |
| 301, 303, 307 e 308 | `transporte/web_mock_tests.rs::matriz_get_3xx_nao_302_recusados_sem_follow` e `formulario_web/tests_envio.rs::post_com_307_ou_308_nao_reenvia_o_formulario` | Recusa tipada, uma tentativa, nenhum follow. |
| MIME inesperado e corpo acima do limite | `web_mock_redirecionamento_mime_e_limite_rejeitados_sem_retry` e `matriz_get_limite_em_stream_sem_content_length` | Recusa do GET com ou sem `Content-Length`. |
| Falha de transporte | `matriz_get_erro_de_transporte_nao_repete_pedido` e `formulario_web/tests_envio.rs::falha_de_rede_nao_repete_o_envio` | Erro tipado sem retry. |
| Origin, Referer e corpo do POST | `formulario_web/tests_envio.rs::closure_recebe_post_corpo_cabecalhos_e_cookie_da_sessao` | Campos preservados do formulário servido na bancada de pesquisa. |
| Charset e ausência de reconstrução de hash | `formulario_web` e testes por operação | Ainda requer assertivas específicas em todos os fluxos; não inferir cobertura apenas da infraestrutura. |

O teste específico `parametro_generico_duplicado_deve_ser_recusado_antes_do_cookie`
está marcado `#[ignore]` por expor um bug documentado na #241, que deve
ser corrigido em PR separada. A matriz acima não equivale à comprovação cruzada integral por operação.
O RED local, o CI e a validação live devem ser relatados separadamente.
Testes de leitura em instalações de treinamento só podem receber PASS após
execução, respeitando o orçamento e as restrições do AGENTS.md.
