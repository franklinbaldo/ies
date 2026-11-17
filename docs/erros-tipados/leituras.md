---
type: Guide
title: Erros das leituras
description: Anexo externo, pesquisa em documentos, versões e filtros da pesquisa
---

# Erros das leituras

Parte do [guia de erros tipados](../erros-tipados.md), separada por responsabilidade.

## Anexo do documento externo

`ErroAnexo` (`src/anexo_documento.rs`, Refs #60 fatia 3) tem uma
erro por fato da resposta de `documento_download_anexo`: sem tipo,
tipo fora da lista observada, cabeçalho repetido, `Content-Disposition`
fora da forma, acima do limite, vazio, exceção do SEI (com status 200 e a
mensagem saneada como apresentação) e as falhas de gravação do parcial.
Status, rede e charset continuam `FalhaTransporte`; a tela de login, a
sessão encerrada de `processo NUP`. Lista no
[contrato](../../contracts/cli/documento-obter-externo.md).

## Pesquisa em documentos

`ies documento buscar` ([contrato](../../contracts/cli/documento-buscar.md))
reaproveita os erros de `processo buscar` e acrescenta, cada uma pelo
fato verificável: na tela, `ErroRegraDoFormularioDePesquisa` com a opção
`#optDocumentos` (`opcao_de_documentos_*`), as caixas de tipo
(`caixa_de_tipo_de_documento_ausente_na_tela`,
`caixas_de_tipo_de_documento_repetidas_na_tela`,
`caixa_de_tipo_de_documento_sem_valor_na_tela`,
`nenhum_tipo_de_documento_marcado_na_tela`) e
`assinante_oculto_preenchido_na_pesquisa_em_documentos`; no 302,
`ErroRedirecionamentoDaPesquisa::IdDocumento*`
(`id_documento_ausente_no_redirecionamento`, `…_repetido_…`,
`…_nao_decimal_…`); na linha, `ErroDocumentoDaPesquisa`, que embute a
recusa comum do link do processo (`ErroResultadoPesquisa`) em vez de
repeti-la. O termo inválido é `TermoDeDocumentoInvalido`
(`termo_de_documento_vazio`, `termo_de_documento_contem_controle`).

## Versões de documento (Refs #88)

`ErroDocumentoObter` ganhou `BarraDoDocumento(ErroBarraDoDocumento)`
(`barra_do_documento_fora_da_forma_observada`,
`barras_do_documento_repetidas_na_arvore`,
`link_das_versoes_repetido_na_barra`, `link_das_versoes_nao_reconhecido`,
`link_das_versoes_de_outro_documento`),
`ListaDeVersoesNaoPublicadaNaArvore`
(`lista_de_versoes_nao_publicada_na_arvore`), `Versoes(ErroVersao)`
(os subcódigos do leitor da lista, inclusive `versao_nao_observada`) e
`VersaoServidaDiferenteDaPedida` (`versao_servida_diferente_da_pedida`).
Com #207, `ConteudoComLinkAssinado` (`conteudo_com_link_assinado`) e, sem
rodapé nem lista, `ListaDeVersoesNaoPublicadaNaArvore` também recusam a
foto de `--saida`. A gravação local é `ErroCopia`
(`src/copia_documento.rs`): `foto_local_divergente`,
`gravacao_foto_falhou` e, na prévia offline de `salvar`,
`base_versao_ausente`, `versao_editada_ausente` e
`versao_editada_igual_a_base` (a prévia não confere a base com o SEI)
([contrato](../../contracts/cli/documento-obter-versoes.md)).

## Filtros da pesquisa (#247)

Em `pesquisa_web::filtros::ErroFiltro`:

- `criterio_nao_oferecido_pela_tela`: o critério pedido não tem controle
  observável, ou depende de ID de autocompletar ainda não resolvido.
- `tipo_nao_oferecido_pela_tela`: a opção exata pedida não existe na lista.
- `controle_de_filtro_ambiguo`: campo ou opção duplicado; não escolher um.
- `valor_do_filtro_diferente_do_solicitado`: nome/valor publicado mudou
  durante a navegação; não publicar página potencialmente não filtrada.
- `atributo_do_filtro_ilegivel`: valor de atributo não interpretável.

Na borda: `data_da_pesquisa_invalida` e `periodo_da_pesquisa_invertido`
são erros de uso (saída 2), avaliados antes da configuração e da rede.

## Inventário normativo dos subcódigos de filtro (#247)

| Subcódigo | Condição |
| --- | --- |
| `criterio_nao_oferecido_pela_tela` | Critério sem controle observável ou sem ID de autocompletar. |
| `tipo_nao_oferecido_pela_tela` | Nome exato de tipo não está na lista servida. |
| `controle_de_filtro_ambiguo` | Mais de um controle ou opção compatível. |
| `atributo_do_filtro_ilegivel` | Atributo HTML não interpretável. |
