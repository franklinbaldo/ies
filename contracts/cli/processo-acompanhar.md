---
type: CommandContract
title: Acompanhamento Especial por prévia e confirmação observada
description: Prévia de inclusão e retirada; confirmação causal com POST único, restrita ao treinamento para retirada
operation: processo.acompanhar
command: ies processo acompanhar NUP [--grupo G] [--observacao T] [--deixar] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo acompanhar NUP` — prévia e confirmação por leitura (Refs #254, #348)

O comando abre a árvore do processo e confere a unidade da sessão.
Lê o link assinado `acompanhamento_gerenciar` publicado na barra da raiz.
O GET pode retornar 302 para `acompanhamento_cadastrar` quando não existe
acompanhamento naquela unidade; este redirecionamento é **dado**, não
seguido automaticamente, e o destino é verificado pelo processo, ação,
origem e assinatura antes de fazer um GET explícito.

Com acompanhamentos, a página publica a tabela com resumo
`Tabela de Acompanhamentos.` e `btnAdicionar`, cuja URL vem da própria
tela. A lista incompleta não prova ausência. O grupo é escolhido pelo
**nome exato** entre as opções habilitadas de `selGrupoAcompanhamento`.
Grupo ausente ou repetido é erro tipado. O nome vazio usa a opção em
branco servida; não se inventa seu valor.

`--deixar` seleciona uma linha pela identidade; na ausência de
`--grupo`, exige no máximo uma. Se nenhum acompanhamento existe,
a prévia informa ausência de efeito. `--observacao` não combina com
`--deixar`. Uma inclusão de grupo já presente informa que nada deve
ser enviado, independentemente da observação.

## Estado desta fatia

- **Prévia:** implementada com leituras reais da interface Web. A saída
  informa `numero`, `unidade`, `operacao`, `grupo`, `observacao`,
  `id_acompanhamento` quando houver, `ja_estava`, `confirmado`,
  `alterado`, `origem:web` e `requisicoes`.
- **Inclusão `--confirmar`:** a tela servida é validada e dela é
  derivada exatamente uma submissão `acompanhamento_cadastrar`, com
  `selGrupoAcompanhamento` oferecido, `txaObservacao`, ocultos e botão
  observados. Um POST no máximo, sem retry e sem seguir 302. Releitura
  explícita do gerenciador prova uma única linha nova com grupo e
  observação pedidos, preservadas as linhas anteriores. Falha no POST
  também exige releitura: se a alteração não puder ser comprovada,
  saída **1** (`AcompanhamentoNaoConfirmado`, sem retry).
  A inclusão já existente resulta em sucesso sem POST.
- **Retirada `--deixar --confirmar`:** restrita a instalações públicas
  de treinamento e ao anti-IES (`sei.test`). A oferta da linha e a action
  assinada `acompanhamento_excluir` são reconhecidas na própria tela
  servida. O envio reescreve somente `hdnInfraItemId`, executa **um POST**
  e faz GET novo do gerenciador: só há confirmação quando exatamente a
  identidade solicitada desapareceu e todas as outras foram preservadas.
  Grupo solicitado já ausente é no-op sem POST. Resposta/estado incerto
  produz saída **1**, sem retry. Falha de transporte/decodificação na
  releitura posterior ao POST mantém o efeito incerto e a causa tipada
  `ReleituraDaRetiradaFalhou`. Uma lista ilegível propaga seu erro de
  leitura; divergência de estado permanece `AcompanhamentoNaoConfirmado`.
  Redirecionamento 302 após excluir a última linha não comprova lista
  vazia sozinho e permanece incerto.
  A comprovação aqui é sintética (CLI ↔ anti-IES ↔ SQLite); confrontos
  semânticos independentes com cassettes e treinamento/live são pendentes.
- **Orçamento:** na bancada sintética sem índice quente, a inclusão
  confirmada usa 3 requisições de abertura + GET gerenciar + GET cadastro
  + POST + GET de verificação = **7 requisições**. Retirada de uma linha
  com outras remanescentes usa 3 requisições de abertura + GET gerenciar
  + POST + GET de verificação = **6 requisições**. O custo de abertura
  real varia com o índice local; nunca há retry.
- **Fonte:** observação histórica do Todos na UnB em 15/10/2026
  (`tests/test_processo_acompanhar_010.py` @323eb01), cassettes
  `processo-acompanhar-*` e as demais gravações dessa operação.
- Testes sintéticos de derivação e transporte integrados; evidência ao vivo e replay ainda pendentes: ver
  [relatório](../../docs/validacao-live/2026-11-05-processo-acompanhar.md).

Os erros (o stderr traz o nome do erro):
`ErroAcompanhar::LinkNaoPublicado`,
`ErroAcompanhar::CadastroDeOutroProcesso`,
`ErroAcompanhar::GruposNaoPublicados`,
`ErroAcompanhar::GrupoNaoOferecido`,
`ErroAcompanhar::GrupoAmbiguo`,
`ErroAcompanhar::ListaNaoPublicada`,
`linha_de_acompanhamento_invalida`,
`ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua`, entre outros
em `src/processo_acompanhar.rs`.

Todos os URLs assinados e cookies ficam privados; nenhuma saída ou
fixture contém credencial real. Nenhuma operação é dirigida à produção.

## Inventário de erros tipados

A coluna é o que aparece no stderr: o erro tipado, com o tipo que o declara.

| Erro (stderr) | Condição |
| --- | --- |
| `ErroAcompanhar::AcompanhamentoNaoConfirmado` | A releitura não demonstrou a alteração exata; incerto |
| `ErroAcompanhar::ReleituraDaRetiradaFalhou(FalhaTransporte)` | Falha de transporte/decodificação na releitura após POST; escrita incerta |
| `ErroAcompanhar::LinkNaoPublicado` | A barra do processo não oferece Acompanhamento Especial |
| `ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua` | Mais de uma linha satisfaz a escolha de remoção |
| `ErroAcompanhar::Barra(ErroAcompanharNaBarra::Barra(ErroBarraDoProcesso::BarraDoProcessoForaDaFormaObservada))` | Barra fora do formato observado |
| `ErroAcompanhar::Barra(ErroAcompanharNaBarra::Barra(ErroBarraDoProcesso::BarrasDoProcessoRepetidasNaArvore))` | A árvore publica mais de uma barra para a raiz |
| `ErroAcompanhar::CadastroDeOutroProcesso` | Identidade do processo divergente no cadastro |
| `ErroAcompanhar::CadastroNaoPublicado` | Formulário de cadastro ausente |
| `ErroAcompanhar::ColunasDeAcompanhamentosInvalidas` | Cabeçalho da lista incompleto ou repetido |
| `ErroAcompanhar::FormulariosDeAcompanhamentoRepetidos` | Mais de um formulário para a operação |
| `ErroAcompanhar::GrupoAmbiguo` | O nome informado corresponde a mais de uma opção |
| `ErroAcompanhar::GrupoNaoOferecido` | Grupo solicitado não é oferecido na tela |
| `ErroAcompanhar::GrupoJaAcompanhado` | O grupo já aparece na lista anterior |
| `ErroAcompanhar::GruposNaoPublicados` | Select de grupos ausente ou sem opções |
| `ErroAcompanhar::HtmlDeAcompanhamentoInvalido` | HTML ilegível ou acima do limite |
| `ErroAcompanhar::IdDeAcompanhamentoRepetido` | A tabela repete a identidade de uma linha |
| `ErroAcompanhar::LinhaDeAcompanhamentoInvalida` | Colunas ou checkbox de uma linha fora da forma |
| `ErroAcompanhar::Barra(ErroAcompanharNaBarra::LinkDeOutroProcesso)` | Link assinado da barra indica outro processo |
| `ErroAcompanhar::Barra(ErroAcompanharNaBarra::LinkDeAcompanhamentoNaoReconhecido)` | Link da barra não segue a forma permitida |
| `ErroAcompanhar::Barra(ErroAcompanharNaBarra::LinksDeAcompanhamentoRepetidos)` | Mais de um link de acompanhamento na barra |
| `ErroAcompanhar::ListaNaoPublicada` | Não existe tabela comprovando a lista |
| `ErroAcompanhar::ListasDeAcompanhamentosRepetidas` | Mais de uma tabela de acompanhamento |
| `ErroAcompanhar::OpcoesDeGrupoDeAcompanhamentoInvalidas` | Option sem valor válido |
| `ErroAcompanhar::OrigemNaoAutorizada` | Origem da sessão inválida |
| `ErroAcompanhar::PedidoDeAcompanhamentoInvalido` | Combinação de opções inválida |
| `ErroAcompanhar::NenhumAcompanhamento` | Não há linha a selecionar para remoção |
| `ErroAcompanhar::RedirecionamentoDeAcompanhamentoInesperado` | Redirect não comprovado para o cadastro da mesma origem |
| `ErroAcompanhar::TelaDeAcompanhamentoInesperada` | Formato ou controle da tela divergente |
| `ErroAcompanhar::UnidadeNaoPublicada` | Unidade da tela não está publicada |
