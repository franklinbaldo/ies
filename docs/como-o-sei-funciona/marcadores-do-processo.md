---
type: SEIObservation
title: Marcadores do processo na barra, na tela de gerenciamento e no Controle
description: Link andamento_marcador_gerenciar na barra do nó do processo, a tela Marcadores do Processo com table#tblMarcadores (uma linha por marcador) e a célula Marcadores da visão detalhada do Controle (um a.ancMarcador por marcador)
---

# Marcadores do processo

Observado na UnB 4.0.9 (treinamento, 04/11/2026), em capturas privadas
`0600` lidas só pela estrutura. Nenhum valor real aparece aqui ou nas
fixtures. Uma instalação não prova as outras.

**Um processo pode ter mais de um marcador.** A captura tem um processo
com dois; a célula do Controle e a tela de gerenciamento mostram os dois,
cada um no seu elemento. Juntar os textos (o que o IES fazia na visão
detalhada) cola os nomes numa string sem separador.

## O link na barra

A barra do nó `PROCESSO` da [árvore](arvore-do-processo.md)
(`Nos[0].acoes = '<literal>';`, a mesma das
[propriedades](propriedades-do-processo.md)) publica um
`<a href="controlador.php?acao=andamento_marcador_gerenciar&…&id_procedimento=N&arvore=1&…&infra_hash=<64 hex>">`
(ícone "Gerenciar Marcador"). Nas árvores capturadas da UnB, o link
aparece nas três árvores completas e falta nas duas de pastas e de
anexado (outra barra). GO e PEN1 têm a mesma ação na barra (cassettes de
escrita de marcador).

## A tela

`GET andamento_marcador_gerenciar` → 200, `iso-8859-1`,
`<title>SEI - Marcadores do Processo <NUP></title>`.

- `form#frmGerenciarMarcador` (`method=post`), `action` assinado com
  `acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_gerenciar&id_procedimento=N`.
- **Escrita na mesma tela**: `button#btnAdicionar` (`location.href` para
  `andamento_marcador_cadastrar`), `button#btnRemover`
  (`acaoRemocaoMultipla()`, que troca o `action` do formulário para
  `andamento_marcador_remover` e o envia), e por linha
  `andamento_marcador_alterar` e `acaoRemover('<id>','<nome>')`. O IES só
  faz o GET; nada disso é enviado nem seguido.
- `table#tblMarcadores` (`summary="Marcadores"`), legenda
  `Marcadores (N registros):` sem faixa, sem paginação visível.
- Cabeçalho: seleção (`#lnkInfraCheck`), `Marcador`, `Texto`, `Usuário`,
  `Data/Hora` (os três com `div.infraDivRotuloOrdenacao`), `Ações`.

Cada `tr.infraTrClara` é **um marcador**, seis `td`:

| Coluna | Forma |
| --- | --- |
| seleção | `a#lnkInfraID-<id>` `name="ID-<id>"` e `input.infraCheckbox` com `value="<id>"` e `title` igual ao nome |
| Marcador | `<a href="#"><img src="svg/marcador_<cor>.svg?11" title="<nome>"/></a>&nbsp;<nome>` |
| Texto | `td#tdAndamentoMarcador<id>`; vazio sem texto |
| Usuário | `a.ancoraSigla` com a sigla (`title` com o nome completo) |
| Data/Hora | `dd/mm/aaaa hh:mm` |
| Ações | alterar e remover (escrita) |

O `<id>` é o do andamento do marcador: o mesmo `#ID-<id>` do link da
célula do Controle. O NUP do `title` e o `id_procedimento` do `action`
identificam o processo.

## A célula do Controle

Na visão detalhada (D) com a coluna Marcadores marcada no Nível de
detalhe, a célula (`td align="left"`, sem `data-label` na 4.0.9) tem uma
âncora por marcador, coladas, sem separador:

```html
<a class="ancMarcador" href="controlador.php?acao=andamento_marcador_gerenciar&…&id_procedimento=N&…&infra_hash=…#ID-<id>"
   onclick="…" onmouseover="return infraTooltipMostrar('<texto>');" onmouseout="…">
  <div><img src="svg/marcador_<cor>.svg?11" class="imagemStatus"/></div><div><nome></div></a>
```

O tooltip tem um só argumento (o texto do marcador, vazio quando não
há). Sem marcador, a célula traz `&nbsp;`. Na captura, oito âncoras em
sete processos; os ids de um processo com dois marcadores são os mesmos
das duas linhas da tela de gerenciamento.

## Efeito

GET de tela de gerenciamento, sem envio. Declarado leitura; nenhuma
mudança de estado observada. Nada a restaurar.

## Não observado

A tela sem marcador (sem tabela ou com `0 registros`), paginação da
tabela, marcadores de outras unidades e outra forma da célula em 5.0.x.
O leitor publica `null` com motivo onde não há observação e recusa formas
novas.
