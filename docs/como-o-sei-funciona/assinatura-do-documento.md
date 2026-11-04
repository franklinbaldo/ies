---
type: SEIObservation
title: Assinar o documento pela tela que a árvore publica
description: Onde a árvore publica "Assinar Documento" (barra do nó e link na tela do documento, em duas formas), a tela frmAssinaturas, a resposta do POST com a senha e a caneta de assinatura na árvore relida, nas sete instalações de treinamento
---

# Assinatura do documento

Observado nas cassettes públicas do Todos (capturas até 20/10/2026), lidas
pela estrutura e nunca executando JavaScript: as gravações
`documento-escritas-*` das sete instalações (UnB 4.0.9, PEN1 4.0.12 nas
cassettes, UFRJ e PEN2 4.1.5, GO 5.0.3, PEN3 e PEN4 5.0.4) e
`documento-mover-cancelar-go`. A gravação não guarda o corpo do POST; os
campos enviados vêm do formulário servido e do script dele (e do Todos,
que envia os mesmos campos com sucesso).

## Onde a ação é publicada

- **Barra do nó** `DOCUMENTO` (`Nos[i].acoes`), nas sete:
  `<a href="#" onclick="assinarDocumento();" …><img … title="Assinar
  Documento"/></a>`. Continua publicada depois de assinado (outro usuário
  pode assinar).
- **A URL** está na tela do documento (`arvore_visualizar` com
  `id_documento`, o a3 do nó), em duas formas:
  - 4.0 (UnB, PEN1 nas cassettes): `function assinarDocumento(){` /
    `parent.infraAbrirJanelaModal('controlador.php?acao=documento_assinar&acao_origem=arvore_visualizar&id_procedimento=P&id_documento=D&arvore=1&…&infra_hash=…',600,450);`
    / `}`;
  - 4.1 e 5.0 (UFRJ, PEN2, GO, PEN3, PEN4): `var linkAssinarDocumento =
    '<mesma URL>';`.

## A tela "Assinatura de Documento"

`GET documento_assinar` → HTML 200 (`SEI - Assinatura de Documento`),
janela modal sem `lnkInfraUnidade`. Abrir não assina: em cada gravação a
tela foi aberta duas vezes e a árvore relida entre elas não mostra
assinatura. `form#frmAssinaturas method="post"` com `action`
`documento_assinar&acao_origem=documento_assinar&acao_retorno=principal&hash_documentos=…`.
Controles, idênticos nas sete (fora `btnCopiarCodigo`, ausente em GO, PEN3
e PEN4):

| Nome | Tipo | Valor servido |
| --- | --- | --- |
| `hdnInfraTipoPagina` | hidden | `2` |
| `btnAssinar` | `button`, `onclick="assinarSenha();"` | não viaja |
| `selOrgao` | select | órgão do usuário `selected` (1 a 63 opções) |
| `txtUsuario` / `hdnIdUsuario` | text / hidden | o usuário da sessão |
| `selCargoFuncao` | select | opção vazia `null` + 2 a 21 cargos, um `selected` |
| `pwdSenha` | password | vazio |
| `btnCopiarCodigo` | `button` | não viaja |
| `hdnFormaAutenticacao` | hidden | vazio; o script põe `S` (senha) ou `C` |
| `hdnLinkRetorno`, `hdnIdBlocos` | hidden | vazios |
| `hdnFlagAssinatura` | hidden | `1` |
| `hdnIdDocumentos` | hidden | o id do documento |

`assinarSenha()` recusa senha vazia, põe `hdnFormaAutenticacao='S'` e,
se `OnSubmitForm()` (órgão, assinante e cargo escolhidos) aceitar, chama
`frmAssinaturas.submit()`. O `inicializar()` escolhe a única opção de cargo
quando só há ela e a vazia. O cargo `selected` é o que a tela sugere.

## A resposta

`POST documento_assinar` → HTML 200, a mesma tela, com o bloco da
autenticação escondido e `inicializar()` começando por
`//se realizou assinatura` e `finalizar();`; o `finalizar()` recarrega a
árvore (`procedimento_visualizar&acao_origem=documento_assinar&montar_visualizacao=1&…`)
e fecha a janela. Na tela servida pelo GET, a linha seguinte ao comentário
é o `if` do cargo. Nenhum 302. A recusa (senha errada) não foi gravada; o
Todos a lê em `divInfraMsg0`.

## Depois: a caneta na árvore

A árvore relida traz, para o documento, uma linha
`NosAcoes[j] = new infraArvoreAcao("ASSINATURA","A<id>","<id>",<a4>,<a5>,"Assinado por:\n<nome>\n<cargo>\n<unidade>","svg/assinatura1.svg?…",true);`
(a4 `javascript:alert(…)` e a5 `null` no 4.0, UFRJ e PEN2; a4 o link
assinado `assinatura_listar` e a5 `"ifrVisualizacao"` em GO, PEN3 e
PEN4), e a barra passa de `editarConteudo('N')` a `editarConteudo('S')`.
Em todo o acervo (1.687 linhas `ASSINATURA`), a dica tem um só assinante
(quatro linhas; o ícone é `assinatura1.svg` ou `assinatura2.svg`).
A página impressa (`documento_imprimir_web`) ganha o parágrafo "Documento
assinado eletronicamente por NOME, CARGO, em DATA…" (texto que muda por
instalação). O nome da dica é o `txtUsuario` da tela e o cargo, o escolhido.

## O que isso implica

A verificação lê a caneta da árvore (núcleo do SEI, mesma forma nas sete),
não o carimbo da impressa. Uma dica com mais de um assinante não foi
observada: é "fora da forma" e a escrita não sai.
