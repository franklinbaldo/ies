---
type: Contract
title: Onde o processo está aberto e natureza dos documentos pela árvore
description: ies processo NUP publica, sem requisição a mais, as unidades onde o processo está aberto ou sobrestado (o .html do nó PROCESSO) e o ícone e o cancelamento de cada documento (a6 a a9 do nó DOCUMENTO), lidos da árvore já conferida
---

# Situação e natureza na árvore de `ies processo NUP` (Refs #59)

Complementa o [contrato da árvore](processo-obter-arvore.md). As duas
leituras vêm da **mesma** árvore já pedida e conferida: **0
requisições a mais** em qualquer caminho, com ou sem `--sem-*`.
Estrutura em
[situação na árvore](../../docs/como-o-sei-funciona/situacao-na-arvore.md);
decisão em [processo.obter](../../docs/decisoes/processo-obter/situacao.md).

## Onde o processo está aberto (`aberto_em`)

Fonte: o literal da linha `Nos[i].html = '<texto>';` do nó `PROCESSO`
(sozinha ou seguida da declaração seguinte), lido pelo leitor de
literais JS da árvore; nada é executado. O `<texto>` é lido pelo `tl`
só no primeiro nível, e cada `<a>` pelo tokenizador de atributos. São
aceitas **só** estas quatro formas, peça por peça, sem sobra:

1. `Processo aberto somente na unidade ` + âncora + `.` + `<br />`; ou
   `Processo aberto somente na unidade ` + âncora +
   ` (atribuído para ` + âncora + `).` + `<br />`.
2. `Processo aberto nas unidades:` + `<br />` + uma ou mais vezes
   (âncora + `<br />`, ou âncora + ` (atribuído para ` + âncora + `)` +
   `<br />`) + um `<br />` final.
3. `Processo não possui andamentos abertos.` + `<br />`: lista vazia.
4. `Processo sobrestado na unidade ` + âncora + `.` + `<br />`: o
   sobrestamento (ver `sobrestado_em`); `aberto_em` não é publicado.

Âncora é `<a>` com exatamente os atributos `alt`, `title` e `class`,
`class` igual a `ancoraSigla`, `alt` igual a `title` (entidades
decodificadas, não vazio, sem espaço nas pontas) e um único filho de
texto não vazio, sem espaço nas pontas, entidade ou tag. O texto é a
`sigla`; o `alt` é o `nome`. Na atribuição, a sigla é o login e o nome o
da pessoa. Sigla de unidade repetida, outra tag, outro texto entre as
peças, comentário ou qualquer outra forma: forma não observada.

```json
"aberto_em": [{"sigla": "Unidade1", "nome": "Unidade1",
               "atribuido_a": {"sigla": "aluno2", "nome": "aluno2"}},
              {"sigla": "ACE", "nome": "Arquivo Central", "atribuido_a": null}]
```

`[]` só para a forma 3. Na forma 4 e fora das quatro, `"aberto_em": null`,
`"aberto_em"` em `nao_lidos` e `aberto_em_nao_lido_motivo`:

| Motivo | Quando |
| --- | --- |
| `raiz_publica_o_vinculo_de_anexacao` | o processo é anexado (`anexado_a` conferido): o `.html` da raiz é o aviso do vínculo |
| `raiz_publica_o_sobrestamento` | forma 4: a frase diz onde está sobrestado, não onde está aberto |
| `html_da_raiz_nao_publicado` | nenhuma linha `.html` para o nó `PROCESSO` |
| `html_da_raiz_fora_da_forma_observada` | qualquer outro texto |

Nunca é falha do comando: a árvore já provou a identidade. `aberto_em`
e `sobrestado_em` saem **sempre** (também com `--sem-documentos`), em
ordem de serviço. O campo `unidade` (a unidade da sessão publicada na
tela) não muda.

## Onde o processo está sobrestado (`sobrestado_em`)

Só na forma 4: lista com a unidade da âncora, `{"sigla", "nome"}`, lida
como as de `aberto_em` (sem atribuição, que a frase não traz).
Em qualquer outra forma, `"sobrestado_em": null`: **não observado**, não
"não sobrestado"; não entra em `nao_lidos` (o que não foi lido é
`aberto_em`, com o motivo). Sobrestamento com atribuição, em várias
unidades ou com outra frase: fora da forma 4, `aberto_em` não lido com
`html_da_raiz_fora_da_forma_observada`.

```json
"aberto_em": null, "aberto_em_nao_lido_motivo": "raiz_publica_o_sobrestamento",
"sobrestado_em": [{"sigla": "Unidade1", "nome": "Unidade1"}]
```

Texto humano, uma linha logo depois da primeira:
`Aberto em: Unidade1 (atribuído a aluno2), ACE.`,
`Sem andamentos abertos.`, `Sobrestado em: Unidade1.` ou
`Aberto em: não lido (<motivo>).`

## Natureza dos documentos

Em cada `DOCUMENTO` (do primeiro nível e de pastas; o
`DOCUMENTO_MOVIDO` não muda), quatro campos novos:

- `icone`: o radical de a7 quando a7, a8 e a9 são o mesmo texto
  `svg/<radical>.svg`, com ou sem `?<decimal>`, e o radical só tem
  minúsculas ASCII, dígitos e `_`. Senão, a leitura da árvore recusa
  (`IconeDoDocumentoForaDaFormaObservada`). Valor **opaco**: o IES
  não traduz `documento_pdf` em "externo" nem `documento_email` em
  "e-mail" (ver a [observação](../../docs/como-o-sei-funciona/situacao-na-arvore.md)).
- `cancelado`: `true` quando o `icone` é `documento_cancelado` **e** a
  dica (a6) começa por `Documento Cancelado:`; `false` quando nenhuma das
  duas ocorre. Com uma só marca a árvore recusa
  (`IconeDeCancelamentoSemMotivoNaDica` ou `DicaDeCancelamentoSemIconeDeCancelado`):
  não há `null` nem "não observado".
- `motivo_do_cancelamento`: com `cancelado: true`, o resto da dica depois
  de `Documento Cancelado:`, sem espaços nas pontas; senão `null`.

- `dica`: a6 decodificada pelo leitor de literais (`\n` vira quebra de
  linha), quando difere do rótulo (a5); igual ao rótulo, `null` (nada a
  acrescentar). Nos e-mails traz `Data de Envio`, `De`, `Para` (ou `Para
  (com cópia oculta)`) e `Assunto`, uma linha cada; no cancelado, a frase
  `Documento Cancelado: <motivo>`. O IES não interpreta a dica: o texto
  sai como servido (decisão do proprietário, 04/11/2026: as instalações de
  treinamento são públicas; revoga a recusa anterior do contrato da
  árvore).

```json
{"id_documento": "D", "rotulo": "Consulta Jurídica 8441007", "numero_sei": "8441007",
 "pasta": null, "icone": "documento_cancelado", "cancelado": true,
 "motivo_do_cancelamento": "erro", "dica": "Documento Cancelado: erro"}
```

Texto humano: `[cancelado: <motivo>]` no fim da linha do documento. Fora do cancelado,
cada linha não vazia da dica sai abaixo do documento, recuada em quatro
espaços.

## `nao_lidos`

Passa a ser, na ordem: `aberto_em` (se não lido), `andamentos`,
`marcadores`, `propriedades` (se não lidos) e `documentos_dos_anexados`.

Validação live: **PASS** na UnB (unidade única, plural com atribuição,
anexado, quente; 0 requisições a mais); cancelado, sobrestado, GO e PEN1
ao vivo **NOT_RUN** ([relatório](../../docs/validacao-live/2026-11-04-processo-obter-situacao.md)).
