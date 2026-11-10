---
type: CommandContract
title: Assinar o documento com a senha do usuário da sessão
operation: documento.assinar
command: ies documento assinar DOC [--cargo TEXTO] [--confirmar]
effect: write-document-signature
implementation: partial
---

# `ies documento assinar DOC [--cargo TEXTO] [--confirmar]` (Refs #174)

Assina o documento **com o usuário da sessão e a senha do login**, uma
vez e por pedido explícito, pela tela "Assinatura de Documento" que a
árvore publica. Estrutura observada em
[assinatura do documento](../../docs/como-o-sei-funciona/assinatura-do-documento.md).
**A assinatura não se desfaz** pela CLI (nem pela tela de assinatura):
o contrato exige o cargo escrito por extenso para confirmar.

Segue as invariantes de escrita do [contrato da CLI](../cli.md): prévia,
confirmação e verificação; escrita não confirmada nunca se repete; saídas 0
sucesso, 1 falha, 2 uso inválido, 3 escrita não confirmada. Só a Web e a mesma
sessão; nenhuma URL montada (só as publicadas pela árvore e pela tela do
documento).

## Entrada

- `DOC`: número SEI, como em [`ies documento DOC`](documento-obter-web.md);
  inválido sai com 2 (erro do clap) antes de qualquer arquivo.
- `--processo NUP` (do substantivo, `ies documento --processo NUP assinar
  DOC`): o caminho de abertura de `documento DOC --processo NUP`.
- `--cargo TEXTO`: o cargo/função **exatamente** como uma opção de
  `selCargoFuncao` da tela (o `value`, igual ao texto servido). Opcional na
  prévia; **obrigatório** com `--confirmar` (sem ele, saída 2 do clap,
  antes de qualquer arquivo).
- `--confirmar`: assina. Sem ele, só prévia.
- **Senha**: só a linha `senha=` do arquivo privado `0600` de
  `IES_CREDENCIAIS_FILE` ([formato](sessao-login.md)), a mesma do login.
  Lida **só** com `--confirmar`, antes da primeira requisição; nunca vem de
  argumento, nunca sai em log, saída, erro ou `Debug`. O cargo não vai ao
  arquivo: o formato das credenciais é fechado (três chaves) e o cargo é
  escolha por documento, não segredo.

## Comportamento

1. **Abre o documento** pelo caminho de `ies documento DOC` (processo
   informado, índice ou pesquisa rápida) **até a árvore conferida**, sem o
   GET do conteúdo.
2. **Lê da árvore**, para o nó `DOCUMENTO` do id aberto: a ação
   `onclick="assinarDocumento();"` da barra (`Nos[i].acoes`), a edição
   (`editarConteudo('N'|'S')`), as assinaturas (`NosAcoes[j] = new
   infraArvoreAcao("ASSINATURA","A<id>","<id>",…,"Assinado por:\n<nome>\n<cargo>\n<unidade>",…)`)
   e o `href` assinado da tela do documento (a3 do nó, `arvore_visualizar`
   do mesmo `id_documento`).
3. **GET da tela do documento** e leitura do link `documento_assinar` do
   mesmo `id_documento`, nas duas formas observadas: `var
   linkAssinarDocumento = '<url>';` (4.1 e 5.0) ou `function
   assinarDocumento(){` / `parent.infraAbrirJanelaModal('<url>',600,450);`
   / `}` (4.0).
4. **GET da tela "Assinatura de Documento"**: abrir a tela não assina
   (nas sete instalações a árvore relida depois do GET não mostra
   assinatura; o Todos a abre também na prévia). Conferida antes de
   decidir (abaixo).
5. **Decide antes de escrever**, nesta ordem:

| Fato lido | Saída | Resultado |
| --- | --- | --- |
| Barra sem `assinarDocumento()` | 1 | `AssinaturaNaoPublicadaNaBarra` |
| Assinaturas da árvore fora da forma observada | 1 | `assinaturas_do_documento_fora_da_forma_observada` |
| Tela do documento ou link de assinar ausente/recusado | 1 | subcódigo do leitor |
| Tela de assinatura fora da forma | 1 | subcódigo da tela |
| O assinante da tela (`txtUsuario`) já assina o documento | 0 | `alterado: false`, `ja_assinado_pela_sessao: true`, sem escrita |
| `--cargo` fora das opções | 1 | `CargoNaoOferecidoPelaTela` (`cargos`) |
| A caneta já mostra outro assinante (dica de dois não observada) | 1 | `DocumentoAssinadoPorOutroUsuario` (`assinaturas`) |
| Tudo conferido | 0 | prévia, ou a escrita |

6. **Sem `--confirmar`**: publica o assinante, o cargo pedido (ou `null`),
   `cargo_da_tela` (a opção `selected`, ou a única, como o `inicializar()`
   da tela), `cargos`, as assinaturas atuais e a edição. **Nenhum POST**.
7. **Com `--confirmar`**: um POST derivado pelo primitivo de formulário
   (`ASSINAR_DOCUMENTO`) e nenhum retry; depois, a verificação por uma
   leitura nova da árvore (o caminho mais barato de `documento DOC`).

## Conferência da tela "Assinatura de Documento"

`form#frmAssinaturas` `method="post"`, `action` com `acao=documento_assinar`
(conferido pelo primitivo). Todo controle enviável precisa estar nesta
lista, senão `campo_nao_previsto_na_tela_de_assinatura`: ocultos
`hdnInfraTipoPagina`, `hdnIdUsuario`, `hdnFormaAutenticacao`,
`hdnLinkRetorno`, `hdnFlagAssinatura`, `hdnIdDocumentos`, `hdnIdBlocos`;
`selOrgao`, `txtUsuario`, `selCargoFuncao`, `pwdSenha` (`type=password`).
Botões `type=button` (`btnAssinar`, `btnCopiarCodigo`) não viajam: o
`assinarSenha()` põe `hdnFormaAutenticacao='S'` e chama `submit()`.

| Fato | Variante ou `onde` (stderr) |
| --- | --- |
| HTML não interpretado | `tela_de_assinatura_nao_interpretada` |
| Sem o formulário | `formulario_de_assinatura_ausente` |
| Controle fora da lista | `campo_nao_previsto_na_tela_de_assinatura` |
| `hdnIdDocumentos` diferente do id aberto | `TelaDeAssinaturaDeOutroDocumento` |
| `hdnIdUsuario` ou `txtUsuario` vazio | `AssinanteNaoPublicadoNaTela` |
| `selOrgao` sem uma opção `selected` | `OrgaoDoAssinanteNaoSelecionado` |
| Sem `pwdSenha` de senha | `campo_de_senha_ausente_na_tela` |

O POST leva, na ordem do documento, os ocultos como servidos (exceto
`hdnFormaAutenticacao=S`), `selOrgao` com a opção `selected`, `txtUsuario`
como servido, `selCargoFuncao=<--cargo>` e `pwdSenha=<senha>`, no charset
da tela. A janela não publica `lnkInfraUnidade`
(`ConferenciaDaUnidade::NaTelaDeOrigemDoLink`).

## Resposta e escrita não confirmada

**Esperada**: HTML 200 da própria tela cujo `inicializar()` começa com
`//se realizou assinatura` seguido de `finalizar();` (sete instalações).
Qualquer outra (rede, tempo, status, 302, login, validação da Infra, HTML
sem essa marca) é `falha_do_envio` e **não** repete a escrita: a leitura de
verificação reconcilia, uma vez.

| Resposta | Assinaturas lidas depois | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | as de antes + uma com o assinante e o cargo | 0 | `alterado: true` |
| Não esperada | idem | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_envio` |
| Qualquer | iguais às de antes | 1 | `AssinaturaNaoAplicada` |
| Qualquer | outra mudança, fora da forma, ou leitura falhou | 1 | `NaoConfirmadoNaAssinatura` (`verificacao`) |

Falha **antes** do POST (abertura, árvore, telas, credenciais, derivação)
não é escrita: saída 1. Antes de repetir à mão, conferir com
`ies documento DOC`/a árvore.

## Efeito

`EfeitoAvaliado::AlteraEstadoDoProcesso { AssinaturaDoDocumento }`: grava
a assinatura do usuário (nome, cargo, unidade) no documento; a árvore passa
a mostrar a caneta e a edição vira `editarConteudo('S')` (editar o
conteúdo depois invalida a assinatura). Não há comando que desfaça.

## Saída

```json
{"numero": "0000101", "id_documento": "21", "rotulo": "Despacho 0000101",
 "processo": {"nup": "00001.000001/2026-01", "id": "10"}, "unidade": null,
 "origem": "web", "confirmado": false, "alterado": false,
 "assinante": "Usuário Teste", "cargo": null, "cargo_da_tela": "Assessor(a)",
 "cargos": ["Assessor(a)", "Presidente"], "ja_assinado_pela_sessao": false,
 "assinaturas": [], "assinaturas_depois": null, "edicao": "nao_assinado",
 "reconciliado_por_leitura": false, "requisicoes": 5}
```

`assinaturas`: `{"nome", "cargo", "unidade"}` da árvore. `edicao`:
`nao_assinado`, `assinado`, `nao_publicada` ou `fora_da_forma`. Nunca URL,
`infra_hash`, cookie ou senha. Falhas: saída 1, stdout vazio e
`Error: <variante de ErroAssinar>` em stderr; `cargos`,
`assinaturas_depois`, `falha_do_envio` e `verificacao` são campos das
variantes que os observaram. A prévia
humana termina com "Para assinar, repita com --cargo e --confirmar".

## Sequência e orçamento

Abertura = a de `documento DOC` sem o GET do conteúdo: 1 GET pelo link da
árvore guardado, 2 pelo link de trabalho guardado; com `--processo` a frio pela linha do Controle, 3 GET;
pela pesquisa rápida, até 4 (Controle, POST da pesquisa, tela, árvore);
uma pasta fechada soma 1. Verificação = a mesma abertura, em geral 1 GET
pelo link da árvore que a abertura guardou.

| Caso | Requisições |
| --- | --- |
| Uso inválido, credenciais ilegíveis com `--confirmar` | 0 |
| Recusa da árvore | abertura |
| Prévia, já assinado, cargo fora, outro assinante | abertura + 2 GET |
| `--confirmar` (qualquer verificação) | abertura + 2 GET + 1 POST + verificação |

Os testes conferem a sequência exata (`--processo` a frio: prévia 5,
escrita 7).

## Fora do escopo

Certificado digital (`hdnFormaAutenticacao=C`), assinar por outro usuário
(o autocompletar de `txtUsuario`), blocos de assinatura (`hdnIdBlocos`),
vários documentos, assinatura externa ([liberação](../evidencias/documento-liberar_assinatura_externa.md)).

## Evidência

[Evidência da operação](../evidencias/documento-assinar.md) (Todos, sete
instalações) e [observação estrutural](../../docs/como-o-sei-funciona/assinatura-do-documento.md);
[decisão](../../docs/decisoes/documento-assinar.md). Live: prévia na PEN1
5.0.4 ([relatório](../../docs/validacao-live/2026-11-04-documento-assinar.md));
escrita **PASS** na PEN1 5.0.4 e na UnB 4.0.9 em 04/11/2026 (prévia 3 req.,
`--confirmar` 5 req., verificação pela caneta sem reconciliação, segunda
prévia `ja_assinado_pela_sessao`;
[relatório](../../docs/validacao-live/2026-11-04-documento-assinar-escrita.md)).
