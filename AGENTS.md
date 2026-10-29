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
