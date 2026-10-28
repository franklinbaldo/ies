---
type: Guide
title: Inventário REST WSSEI
description: Endpoints presentes no OpenAPI oficial fixado por commit
---

# Inventário de endpoints do WSSEI REST

Fonte: [OpenAPI JSON no commit c6bfce09](https://github.com/pengovbr/mod-wssei/blob/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.json).
SHA-256 do JSON: `f2186a89d3f2bccd160d925d44be1c0e74926869db34a16f1b34015125865b13`.
Este índice contém método/rota/operationId, não os parâmetros e respostas completos.
Nem todo endpoint está disponível numa instalação real. O OpenAPI original é
autoritativo para a especificação publicada; a instalação precisa ser verificada.

**OpenAPI 3.0.0; 127 paths; 127 operações; 34 schemas.**

## acompanhamentoespecial (1)

- `GET /acompanhamentoespecial/listar` — `listarAcompanhamentoEspecial`

## anotacao (1)

- `POST /anotacao/` — `cadastrarAnotacao`

## assinante (2)

- `GET /assinante/listar` — `listarAssinantes`
- `GET /assinante/orgao` — `listarOrgaosAssinante`

## atividade (2)

- `GET /atividade/listar` — `listarAtividades`
- `POST /atividade/lancar/andamento/processo` — `lancarAtividadeProcesso`

## autenticar (1)

- `POST /autenticar` — `autenticar`

## bloco (27)

- `GET /bloco/assinatura/pesquisar` — `pesquisarBlocoAssinatura`
- `GET /bloco/interno/pesquisar` — `pesquisarBlocoInterno`
- `POST /bloco/assinatura/criar` — `cadastrarBlocoAssinatura`
- `POST /bloco/interno/criar` — `cadastrarBlocoInterno`
- `POST /bloco/assinatura/{bloco}/alterar` — `alterarBlocoAssinatura`
- `POST /bloco/interno/{bloco}/alterar` — `alterarBlocoInterno`
- `POST /bloco/assinatura/excluir` — `excluirBlocoAssinatura`
- `POST /bloco/interno/excluir` — `excluirBlocoInterno`
- `POST /bloco/assinatura/concluir` — `concluirBlocoAssinatura`
- `POST /bloco/interno/concluir` — `concluirBlocoInterno`
- `POST /bloco/assinatura/{bloco}/reabrir` — `reabrirBlocoAssinatura`
- `POST /bloco/interno/{bloco}/reabrir` — `reabrirBlocoInterno`
- `POST /bloco/assinatura/{bloco}/retornar` — `retornarBlocoAssinatura`
- `POST /bloco/assinatura/{bloco}/disponibilizar` — `disponibilizarBlocoAssinatura`
- `POST /bloco/assinatura/{bloco}/disponibilizacao/cancelar` — `cancelarDisponibilizacaoBlocoAssinatura`
- `POST /bloco/{bloco}/anotacao` — `cadastrarAnotacaoBloco`
- `POST /bloco/assinatura/anotacao/cadastrar` — `cadastrarAnotacaoBlocoAssinatura`
- `POST /bloco/interno/anotacao/cadastrar` — `cadastrarAnotacaoBlocoInterno`
- `POST /bloco/assinatura/anotacao/alterar` — `alterarAnotacaoBlocoAssinatura`
- `POST /bloco/interno/anotacao/alterar` — `alterarAnotacaoBlocoInterno`
- `POST /bloco/assinatura/{bloco}/assinar` — `assinarBlocoAssinatura`
- `POST /bloco/assinatura/{bloco}/documentos/retirar` — `retirarDocumentosBlocoAssinatura`
- `POST /bloco/assinatura/{bloco}/documentos/incluir` — `incluirDocumentoBlocoAssinatura`
- `POST /bloco/interno/{bloco}/processos/incluir` — `incluirProcessoBlocoInterno`
- `POST /bloco/interno/{bloco}/processos/retirar` — `retirarProcessoBlocoInterno`
- `GET /bloco/assinatura/{bloco}/documentos/listar` — `listarDocumentosBlocoAssinatura`
- `GET /bloco/interno/{bloco}/processos/listar` — `listarProcessosBlocoInterno`

## contato (2)

- `GET /contato/pesquisar` — `pesquisarContato`
- `POST /contato/criar` — `cadastrarContato`

## contexto (1)

- `GET /contexto/listar/{orgao}` — `listarContextosOrgao`

## documento (23)

- `GET /documento/{documento}/interno/visualizar` — `visualizarDocumento`
- `GET /documento/interno/consultar/{protocolo}` — `consultarDocumentoInterno`
- `POST /documento/linkedicao` — `linkEdicao`
- `GET /documento/baixar/anexo/{protocolo}` — `baixarAnexo`
- `GET /documento/tipo/template` — `pesquisarTipoTemplateDocumento`
- `GET /documento/listar/{procedimento}` — `listarDocumentosProcesso`
- `GET /documento/secao/listar` — `listarSecoesDocumento`
- `POST /documento/ciencia` — `darCienciaDocumento`
- `POST /documento/{procedimento}/externo/criar` — `cadastrarDocumentoExterno`
- `POST /documento/externo/{documento}/alterar` — `alterarDocumentoExterno`
- `POST /documento/{procedimento}/interno/criar` — `cadastrarDocumentoInterno`
- `POST /documento/incluir` — `incluirDocumento`
- `POST /documento/interno/{documento}/alterar` — `alterarDocumentoInterno`
- `POST /documento/assinar` — `assinarDocumento`
- `GET /documento/{documento}/bloco/assinatura/listar` — `listarBlocosAssinaturaDocumento`
- `GET /documento/externo/consultar/{protocolo}` — `consultarDocumentoExterno`
- `GET /documento/assunto/sugestao/{serie}/listar` — `listarSugestoesAssuntoPorTipoDocumento`
- `GET /documento/tipoconferencia/pesquisar` — `pesquisarTipoConferencia`
- `GET /documento/tipo/pesquisar` — `pesquisarTipoDocumento`
- `GET /documento/listar/assinaturas/{documento}` — `listarAssinaturasDocumento`
- `GET /documento/listar/ciencia/{protocolo}` — `listarCienciasDocumento`
- `POST /documento/assinar/bloco` — `assinarDocumentos`
- `POST /documento/secao/alterar` — `alterarSecaoDocumento`

## grupoacompanhamento (4)

- `GET /grupoacompanhamento/listar` — `listarGrupoAcompanhamento`
- `POST /grupoacompanhamento/cadastrar` — `cadastrarGrupoAcompanhamento`
- `POST /grupoacompanhamento/excluir` — `excluirGrupoAcompanhamento`
- `POST /grupoacompanhamento/{grupoacompanhamento}/alterar` — `alterarGrupoAcompanhamento`

## hipoteseLegal (1)

- `GET /hipoteseLegal/pesquisar` — `pesquisarHipoteseLegal`

## marcador (10)

- `GET /marcador/pesquisar` — `pesquisarMarcador`
- `GET /marcador/cores/listar` — `listarCores`
- `POST /marcador/criar` — `cadastrarMarcador`
- `POST /marcador/excluir` — `excluirMarcador`
- `POST /marcador/desativar` — `desativarMarcador`
- `POST /marcador/reativar` — `reativarMarcador`
- `POST /marcador/{marcador}/alterar` — `alterarMarcador`
- `POST /marcador/processo/{protocolo}/marcar` — `marcarProcesso`
- `GET /marcador/processo/{protocolo}/consultar` — `consultarMarcadorProcesso`
- `GET /marcador/processo/{protocolo}/historico/listar` — `historicoMarcadorProcesso`

## observacao (1)

- `POST /observacao/` — `cadastrarObservacao`

## orgao (1)

- `GET /orgao/listar` — `listarOrgaos`

## processo (38)

- `GET /processo/assunto/pesquisar` — `pesquisarAssunto`
- `GET /processo/assunto/sugestao/{tipo-procedimento}/listar` — `listarSugestaoAssuntosProcesso`
- `GET /processo/{protocolo}/ciencia/listar` — `listarCienciasProcesso`
- `GET /processo/listar/meus/acompanhamentos` — `listarMeusAcompanhamentos`
- `GET /processo/listar/acompanhamentos` — `listarAcompanhamentos`
- `GET /processo/{procedimento}/credenciamento/listar` — `listarCredenciamentoProcesso`
- `GET /processo/pesquisar` — `pesquisarGeral`
- `GET /processo/listar` — `listarProcessos`
- `GET /processo/tipo/listar` — `listarTipoProcesso`
- `GET /processo/{protocolo}` — `consultarProcesso`
- `GET /processo/{protocolo}/consultar/atribuicao` — `consultarAtribuicaoProcesso`
- `GET /processo/verifica/acesso/{protocolo}` — `verificarAcessoProcesso`
- `GET /processo/{protocolo}/interessados/listar` — `listarInteressadosProcesso`
- `GET /processo/acompanhamento/consultar` — `consultarAcompanhamentoProcesso`
- `POST /processo/{protocolo}/sobrestar/processo` — `sobrestarProcesso`
- `POST /processo/{procedimento}/credenciamento/conceder` — `concederAcesso`
- `POST /processo/{procedimento}/credenciamento/renunciar` — `renunciaAcesso`
- `POST /processo/{procedimento}/credenciamento/cassar` — `cazzarAcesso`
- `POST /processo/enviar` — `enviarProcesso`
- `POST /processo/concluir` — `concluirProcesso`
- `POST /processo/receber` — `receberProcesso`
- `POST /processo/identificacao/acesso` — `acessoProcesso`
- `POST /processo/acompanhar` — `acompanharProcesso`
- `POST /processo/acompanhamento/alterar` — `alterarAcompanhamentoProcesso`
- `POST /processo/reabrir/{procedimento}` — `reabrirProcesso`
- `POST /processo/atribuir` — `atribuirProcesso`
- `POST /processo/{protocolo}/remover/atribuicao` — `removerAtribuicaoProcesso`
- `POST /processo/{protocolo}/cancelar/sobrestamento` — `cancelarSobrestamentoProcesso`
- `POST /processo/acompanhamento/{acompanhamento}/excluir` — `excluirAcompanhamento`
- `POST /processo/{procedimento}/ciencia` — `darCienciaProcesso`
- `POST /processo/criar` — `cadastrarProcesso`
- `POST /processo/{protocolo}/alterar` — `alterarProcesso`
- `POST /processo/agendar/retorno/programado` — `agendarRetornoProgramado`
- `GET /processo/listar/sobrestamento/{protocolo}` — `consultarSobrestamentoProcesso`
- `GET /processo/listar/unidades/{protocolo}` — `listarUnidadesProcesso`
- `GET /processo/tipo/template` — `consultarTipoTemplate`
- `GET /processo/consultar/{id}` — `detalharProcesso`
- `GET /processo/consultar` — `consultarProcessoProtocoloFormatado`

## protocolomodelo (2)

- `GET /protocolomodelo/grupo/listar` — `listarGrupoModelosDocumento`
- `GET /protocolomodelo/listar` — `listarModeloDocumento`

## serie (1)

- `GET /serie/externo/pesquisar` — `pesquisarTipoDocumentoExterno`

## unidade (3)

- `GET /unidade/pesquisar` — `pesquisarUnidade`
- `GET /unidade/outras/pesquisar` — `pesquisarOutrasUnidades`
- `GET /unidade/textopadrao/interno/pesquisar` — `pesquisarTextoPadraoInterno`

## upload (1)

- `GET /upload/parametros` — `parametrosUpload`

## usuario (4)

- `POST /usuario/alterar/unidade` — `alterarUnidadeUsuario`
- `GET /usuario/listar` — `listarUsuarios`
- `GET /usuario/pesquisar` — `pesquisarUsuario`
- `GET /usuario/unidades` — `retornarUnidadesUsuario`

## versao (1)

- `GET /versao` — `versao`
