---
type: CommandContract
title: Ler documento SEI
operation: documento.obter
command: ies documento DOC
effect: read
implementation: stub
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Ler documento SEI

`ies documento DOC [DOC ...]` lê um ou mais documentos pelo número SEI
visível. A implementação atual permanece um stub sem acesso à rede.

## Entradas

- `DOC [DOC ...]`: um ou mais números de documento.
- `--processo NUP`: contexto opcional, quando informado.
- `--versao N`: índice positivo de versão histórica solicitado; continua indisponível sem árvore e sessão Web verificadas. Não implica listagem pública.
- `--sem-copia`: leitura sem preparação de cópia local; hoje não altera o stub.
- `--saida DIR`: diretório de saída para conteúdo baixado.
- `--ocultar-inacessiveis`: não expor documentos que a sessão não pode acessar.
- `--com-propriedades`: inclui os metadados do cadastro do documento.
- `--json` e `--unidade SIGLA`: opções globais.

## Saída prevista

`LoteConteudosDocumento` representa os conteúdos e metadados obtidos.
No Todos, múltiplos documentos de um mesmo processo podem ser agrupados
num ZIP quando a própria interface do SEI oferece essa operação.
Não presumir essa oferta em instalações que não a apresentam.

## Falhas e evidências

O shim de entrada interpreta `DOC` como número SEI composto apenas por\ndígitos ASCII, preserva zeros à esquerda e não consulta a instalação.\nEntrada inválida retorna `entrada_invalida` com subcódigo tipado e saída 2.\nUma entrada válida ainda retorna `nao_implementado` e
código 1. Arquivo inacessível, ausência de documento e sessão inválida
precisarão de contratos de erro específicos antes de implementar HTTP.

[documento.obter](../evidencias/documento-obter.md) e
[formulários e payloads](../../docs/como-o-sei-funciona/formularios.md)
fornecem evidências de partida. Não inferir autorização ou saída sem prova.

## Recorte adicional: histórico de versões reproduzido offline

A ferramenta auxiliar `ies replay CENARIO --documento --historico-versoes`
aceita número SEI solicitado, ID interno explicitamente observado,
resposta GET `documento_versao_listar` de cassette conferida e,
opcionalmente, `--versao-documento N`. Apresenta somente maior versão
efetivamente observada e metadados mínimos da versão selecionada;
**não implementa** `ies documento DOC --versao N` nem cria
`listar-versoes`. A relação entre número SEI e ID interno permanece
não comprovada e é exposta como `numero_documento_confirmado: false`.
Links assinados não são publicados nem executados. Refs #88.

