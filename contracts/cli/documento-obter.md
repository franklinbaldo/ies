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
