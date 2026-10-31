---
type: Instructions
title: Instruções para agentes do IES
description: Disciplina de desenvolvimento e validação do projeto Rust
---

# Instruções de desenvolvimento

O [contrato da CLI](contracts/cli.md) é a interface pública primária.
Não reintroduza `agir`, `Action` ou `PageModel` públicos.
O Todos é oráculo de comportamento, **não arquitetura a ser portada**.
Antes de implementar transporte de uma operação, consulte os
[contratos oficiais REST OpenAPI e SOAP WSDL](docs/interfaces/README.md),
compare as [alternativas REST, SOAP e Web](docs/arquitetura/0001-rotas-por-evidencia.md)
e siga o [plano experimental](docs/experimentos/comparar-acessos-sei.md).
Não adivinhe disponibilidade de módulo, token ou equivalência entre APIs.
**OpenAPI público não comprova REST funcional na instalação; WSDL SOAP
pode exigir credenciais até para leitura.** Antes de qualquer teste live,
consulte a [política de acesso por instalação](docs/interfaces/acesso-por-instalacao.md),
obtenha autorização e jamais faça probing de endpoints protegidos ou
publique credenciais. Não classifique 401/403/404 como ausência de API
sem diagnóstico autorizado.
Uma falha ou timeout de escrita nunca autoriza troca automática de backend.
Uma operação com `CommandContract.implementation` diferente de `stub`
exige uma [decisão OKF por operação](docs/decisoes/unidade-listar-processos.md):
alternativas concretas, escolha restrita ao escopo, trade-offs, evidências,
testes existentes, lacunas e condições de revisão. O gate `check_design`
confere a ligação ao contrato antes do push; ele **não certifica o mérito**
da escolha. Não crie fichas para os 129 itens ainda não implementados.
Não invente operações pela inspeção de telas: preserve apenas comportamentos
observados e demonstrados nas instalações reais de treinamento do SEI.

## Contract-First TDD: sequência obrigatória de desenvolvimento

Toda nova capacidade começa no [contrato e nos shims tipados](docs/metodo-contract-first.md):
**CONTRACT** (contrato OKF, domínio, erros específicos e shim compilável),
**RED** (testes comportamentais e de variantes tipadas, executados com
falha esperada antes do miolo), **GREEN/REFACTOR** (implementação mínima,
refatoração e gates) e **VALIDATED** (ensaio offline, depois nas instalações
de treinamento autorizadas, idealmente várias em paralelo quando seguro).

Não declarar PASS sem execução. Usar `NOT_RUN` com o bloqueio explícito
quando faltarem instalação, sessão, permissões ou coletor. Quando não houver
credenciais de gravação/publicação de cassettes, entregar roteiro/script
reproduzível para outra máquina; isso não autoriza capturar dados reais
nem dispensa distinguir capacidade offline da live.

## Login Web: referência comportamental verificada no Todos

Antes de criar login, sessão ou bootstrap Web no IES, consulte o código atual
do [Todos](https://github.com/franklinbaldo/todos/blob/6127bd38b9182108c9b0922d6796493e338e5f05/src/todos/sei/session.py),
a [observação do login](https://github.com/franklinbaldo/todos/blob/6127bd38b9182108c9b0922d6796493e338e5f05/docs/como-o-sei-funciona/02-sessao-e-login.md)
e a [matriz de instalações](https://github.com/franklinbaldo/todos/blob/6127bd38b9182108c9b0922d6796493e338e5f05/docs/sei-multi-instalacao.md).
São **evidências de comportamento**, não arquitetura Python para portar,
nem prova de que um login continua funcionando hoje.

- A raiz HTTPS da instalação pode redirecionar para SIP ou para um
  **módulo próprio** de login (Goiás). Interpretar a tela real, sem
  adivinhar endpoint, órgão, sistema, campo ou permissões; nunca tratar
  HTTP 200 de login como autenticação concluída.
- No formulário observado, preservar os campos ocultos e a seleção
  efetiva do órgão/contexto; a ação do formulário é relativa à **URL
  da tela servida**. O JavaScript do `onsubmit` pode determinar
  `hdnAcao`, e o botão submit efetivo também participa da requisição.
  O transporte precisa respeitar a codificação, Origin, Referer e
  cookies relevantes observados. Não basta enviar usuário e senha.
- Somente confirmar login após a navegação autorizada chegar à ação
  `procedimento_controlar` com URL assinada da mesma sessão e contexto
  validado. Algumas contas chegam primeiro ao Painel de Controle:
  seguir apenas o link do menu que a tela realmente publicou.
  Captcha, 2FA, formulário ambíguo, órgão não identificado ou
  credencial recusada exigem **falha tipada específica**, sem insistência
  automática ou adivinhação.
- Sessões reutilizadas precisam manter cookies e URLs assinadas da
  **mesma geração**, com escopo por instalação, identidade e unidade.
  O Todos usa Keyring ou arquivo privado `0600` opt-in e lock de login
  entre processos. IES pode escolher solução Rust mais simples;
  não persistir capacidades assinadas em logs ou artefatos públicos,
  nem usar link antigo após renovar sessão ou trocar unidade.
- O Todos já demonstrou diferenças entre UnB, UFRJ, Goiás e PEN1–PEN4,
  incluindo login próprio de Goiás, seleção de órgão e versões.
  Testar por combinação instalação + conta autorizada + unidade +
  operação, primeiro offline e depois em treinamento quando permitido.
  Reproduzir as condições observadas, não generalizar uma instalação.

**Critério de validação:** Contract-First TDD com shims e erros tipados;
RED para campos omitidos, contexto errado, sessão expirada e recusa;
GREEN local; por fim ensaio autorizado e reproduzível, com orçamento
de requisições e relato PASS/FAIL/NOT_RUN. Em 31/10/2026 houve GET
HTTP 200 das páginas públicas de login da UnB, UFRJ e Goiás, mas **não
foi comprovada sessão autenticada**; tentativas bloqueadas por controles
de segurança não devem ser repetidas nem redirecionadas por outro canal.
Não acessar produção, nem incluir senhas, cookies ou `infra_hash`
em commits, cassettes públicas ou saídas de diagnóstico.

## Testes e spikes nas instalações de treinamento

**Use instalações SEI de treinamento ou homologação para testes e spikes**.
Comece pelas cassettes OKF e replays offline; quando precisar observar
comportamento real, escolha uma instalação de treinamento autorizada e
registre versão, identidade, unidade, operação, requisições e efeitos.
Não use o SEI de produção para desenvolvimento, experimentação,
descoberta de endpoints ou testes que possam alterar estado.

Preferir os ambientes de treinamento já documentados no acervo;
um cenário reproduzido em uma instalação não comprova paridade nas
outras. Compare variantes reais de HTML e comportamento entre versões,
sem impor uma única gramática de protocolo. Antes de acesso autenticado,
confira a política por instalação e a autorização da identidade.

Uma requisição GET não é automaticamente livre de efeitos:
abrir/visualizar processo pode registrar recebimento. POST de pesquisa,
paginação ou troca de unidade exige avaliação explícita do efeito,
mesmo quando usado para consulta. Se faltar autorização ou evidência,
mantenha o spike offline e o comando live bloqueado; não use
produção como substituto. Faça **testes de ablação de etapas** por operação: parta do fluxo\nreproduzido, remova uma requisição ou etapa por vez e compare\nresultado de domínio, identidade/unidade, permissões, efeitos e erros.\nRegistre no OKF o fluxo de referência, a etapa removida, as condições\nde ensaio, os resultados positivos/negativos e o orçamento de requests.\nAblação de leitura não autoriza suprimir verificação necessária de\nescrita ou repetir uma ação de efeito incerto. Só mantenha a remoção\nquando testes em treinamento demonstrarem equivalência para o escopo.\nMeça o número de requisições por operação e elimine consultas duplicadas\nou especulativas.

## Prioridade de interfaces do SEI

**Web é a via principal e o alvo de compatibilidade nacional do IES.**
A interface Web faz parte do uso normal das instalações SEI; implementar
os fluxos Web observados e autorizados tem precedência sobre desenvolver
clientes opcionais. Isso não prova que um fluxo ou versão específica
seja compatível: validar por instalação, com cassettes e testes seguros.

**REST WSSEI tem prioridade baixa.** É um módulo opcional, ausente em
muitas instalações. Só implementar quando houver benefício concreto,
escopo pequeno e esforço marginal baixo, sem atrasar funcionalidades
Web. Não condicionar `processo`, `documento`, `unidade`, `setup`
ou a compatibilidade principal à existência de REST, token ou módulo.
REST nunca é fallback automático de uma escrita.

**SOAP não é prioridade de implementação.** Pode exigir autenticação
própria e credenciais distintas da sessão Web. Não pressupor que o
cookie da interface Web autoriza SOAP, nem tentar extrair ou reutilizar
credenciais sem suporte explícito e autorização. Manter contratos SOAP
apenas como referências, salvo necessidade concreta e aprovada.

As três interfaces não são equivalentes por definição. Ao avaliar
alternativas por operação, documentar brevemente REST e SOAP quando
pertinente, mas não transformar essa comparação em trabalho obrigatório
de implementação ou em bloqueio para evoluir a interface Web.

Todo Markdown autoral deve ser OKF conforme a [política de CI](docs/ci.md), com Specs e schemas em [`.okf/`](.okf/specs/spec.md).
Nenhum arquivo texto pode ultrapassar 20.000 bytes, inclusive arquivos
de código, testes e lockfiles. Divida por responsabilidade, não minifique.

`xtask` contém apenas verificadores de infraestrutura, sem dependências
externas. Não importe o runtime IES para os hooks de tamanho/OKF,
nem reintroduza um gate `cargo build` redundante com os testes.

Antes de enviar qualquer PR:

```sh
uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files
```

Ative os hooks locais com
`uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.
O [GitHub Actions](.github/workflows/ci.yml) executa esses mesmos gates em
PRs e pushes na `main`, com permissões apenas de leitura.
Não crie um segundo conjunto independente de checks.

## Documentação OKF e código

`Cassette`, `OperationEvidence` e `SEIObservation` são categorias
de **documentos OKF**, não classes nem serviços do runtime.
Não criar um subsistema de pesquisa nem uma camada de observação.
O adaptador da interface HTML chama-se `web`, não `web-observed`.
A prova `completa` indica somente cobertura das posições declaradas,
não um snapshot transacional do SEI.

## Evidências antes de implementar

Leia o [índice de evidências por operação](contracts/evidencias/README.md),
com as referências `cenario#passo` da matriz do Todos.
Leia o [catálogo OKF de cassettes](docs/cassettes.md) e o conhecimento
[como o SEI funciona](docs/como-o-sei-funciona/README.md).
Um card por revisão aponta para o ZIP original no Internet Archive.
Reproduza offline a evidência antes de implementar; o nome da cassette
não comprova um efeito nem autoriza escrita live.

Antes de alterar um comando de leitura, consulte seus [contratos OKF](contracts/cli.md)
e as cassettes da operação. A [leitura da caixa](docs/unidade-primeira-leitura.md)
usa replay offline (inclusive páginas já capturadas) ou GET com sessão
existente; **não automatiza POST de paginação**, login nem escrita.
Nunca passe URLs assinadas ou cookies como argumentos de processo:
use arquivos privados `0600` e nunca os inclua em logs, commits ou
capturas. O GET exige `--origem-file` independente, identificado
por `Instalacao` (host + caminho-base HTTPS); essa identidade deve
corresponder à URL **antes** de ler/enviar cookies.
`VersaoSei` é metadado declarado opcional, não autorização nem
predição de compatibilidade. Toda
resposta HTTP tem limite de 8 MB durante o recebimento, não apenas
depois de receber o corpo inteiro. Não seguir redirects.

Em replay multipágina, conferir `lnkInfraUnidade`,
`hdnTipoVisualizacao` e hashes de critérios das duas tabelas.
`--unidade` é pré-condição conferida na tela, não um rótulo ecoado
na saída. Campos de contexto não pertencem ao domínio e não
provam snapshot transacional.

## Nomes precisos e erros tipados

Siga [a disciplina de erros tipados](docs/erros-tipados.md): cada falha
observável tem variante específica de `enum` Rust, com dados tipados e
subcódigo estável. Mensagens são apresentação, nunca o mecanismo de
classificação. Não crie erros genéricos com mensagens variáveis nem
coloque falhas de adaptadores nas entidades administrativas.

Nomeie cada erro pelo **fato verificável**, indicando o objeto e a
condição concreta: prefira `NumeroDoHistoricoDiferenteDoSolicitado`
a `ProcessoDivergente`. Evite `Ambiguo`, `Divergente`,
`Inconsistente` e `Invalido` sem especificar a observação ou regra
violada. Se duas condições são diferentes e exigem diagnósticos diferentes,
nomeie-as separadamente e teste cada uma. Preserve identificadores opacos
como o NUP sem inferir máscara, pontuação ou zeros à esquerda.

Não introduza `completa: bool` nem estado de consulta em entidades.
Paginação pertence ao adaptador; o contrato da operação determina o
sucesso ou a falha, sem inventar propriedades do domínio.

## Domínio tipado mínimo e saída

O Web parser devolve `CaixaDeEntrada` tipada e não recebe unidade nem origem.
`Tabela<Processo>` armazena somente dados observados; os intervalos
são transitórios da paginação. Não introduza `Completude` ou
`completa: bool` como campos persistidos. A borda de apresentação não publica `completa` nem afirma que os dados\nrepresentam um snapshot da unidade. A verificação de intervalos é técnica.

O contrato JSON v2 é verificado por
[golden test](tests/fixtures/caixa-v2.json). Alterações intencionais
de saída exigem atualizar o golden e o contrato OKF na mesma PR.

Não registre segredos, HTML autenticado ou URLs assinadas em commits ou logs.
