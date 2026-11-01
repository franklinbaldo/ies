---
type: ImplementationDecision
title: GET opt-in de documento por URL assinada privada
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: um GET de documento_visualizar com link privado fornecido pelo operador
selected_strategy: web-signed-readonly
alternatives: web-signed-readonly,rest-wssei,soap-official
evidence_level: referencias-historicas-e-mocks-locais
live_validation: nao-realizada-neste-recorte
---

# Consulta Web limitada de documento SEI

## Alternativas consideradas

- **web-signed-readonly:** um GET explicitamente solicitado, de link
  observado da mesma instalação/sessão, com identidade conferida na resposta.
- **rest-wssei:** módulo opcional e token não demonstrados; não usar como
  substituto automático da Web.
- **soap-official:** credencial e equivalência desconhecidas; sem prioridade.

## Comparação

A via Web usa o link efetivamente publicado pela instalação, mas a origem
confere apenas o host e o caminho-base, não a atualidade da sessão. REST e
SOAP são opcionais e não podem ser tratados como equivalentes sem provas.

## Decisão

Validar URL HTTPS assinada para `documento_visualizar`, ID interno decimal,
hash, domínio e caminho-base antes de ler o cookie. Usar apenas a sessão
existente, não seguir redirects nem repetir GET/POST. Verificar MIME,
status, login, título SEI, número visível exato e corpo antes de publicar.
O link e a sessão pertencem à mesma geração; não confirmar frescura
automaticamente, nem construir links por interpolação de hashes.

O GET pode registrar recebimento/visualização. A CLI exige também
reconhecimento explícito `--autorizar-efeito-de-leitura` **antes** de ler
arquivos da sessão ou executar a requisição, mas isso não comprova
permissão institucional nem vincula automaticamente cookie e URL assinada.
Sua execução depende de autorização de efeitos na instalação de treinamento. Nenhum teste live,
login ou escrita SEI é autorizado por esta decisão isoladamente.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-obter.md)
- [Matriz](../../contracts/evidencias/documento-obter.md)
- [Testes de comando](../../src/documento_remoto_cli/tests.rs)
- [Testes Rust da operação](../../tests/documento_html_privado_export.rs)
- [Testes de transporte](../../src/documento_transporte.rs)
- [Política de acesso](../interfaces/acesso-por-instalacao.md)

## Limitações e revisão

Não implementa resolução do documento pela árvore, versões, assinatura,
consulta geral ou login. Revisar após mock Rust com orçamento de uma
requisição e testes live autorizados; nenhum resultado offline certifica
acesso nacional ao SEI.
