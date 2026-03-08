# Plan: Entidade de Inflação Anual com Índice de Multiplicação

## TL;DR
Criar uma nova entidade `AnnualInflation` em arquivo separado que armazene uma lista de pares ano/inflação (parametrizáveis via construtor), implementar um método `calculate_multiplier()` que retorne o índice de multiplicação entre dois anos (com validação de anos ausentes e arredondamento aritmético em 4 casas decimais), e integrar ao módulo público em lib.rs. A entidade será serializável em JSON seguindo os padrões já usados no projeto.

## Decisões Tomadas
- Dados parametrizáveis no construtor (máxima flexibilidade)
- Retornar erro (Result) se anos não estiverem presentes na lista
- Adotar arredondamento aritmético com 4 casas decimais
- Serialização habilitada (Serialize/Deserialize)
- Usar `Decimal` (não `f64`) para precisão financeira consistente
- Enum customizado `InflationError` segue padrão de `ValidationError` em budget.rs

## Steps

### 1. Criar novo arquivo `src/inflation.rs` com estruturas base

Implementar:
- Struct `AnnualInflationEntry` (ano: u32, inflação: Decimal) com derives `Serialize`, `Deserialize`, `Debug`, `Clone`
- Struct `AnnualInflation` com campo `entries: Vec<AnnualInflationEntry>`
- Enum `InflationError` com variante `YearNotFound(u32)` implementando `std::error::Error`

### 2. Implementar construtor e métodos

- `AnnualInflation::new(entries: Vec<(u32, String)>) -> Result<Self, InflationError>` 
  - Converte strings de inflação em Decimal
  - Valida que todos os valores estão presentes

- `calculate_multiplier(&self, start_year: u32, end_year: u32) -> Result<Decimal, InflationError>`
  - Validar que ambos os anos existem na lista
  - Iterar pelos anos no intervalo, acumulando multiplicações: `(1 + inflação/100)`
  - Arredondar resultado para cima (ceil) com 4 casas decimais

### 3. Vincular em `src/lib.rs`

- Adicionar `mod inflation;` (ou `pub mod`)
- Opcionalmente re-exportar `pub use inflation::{AnnualInflation, InflationError};`

### 4. Adicionar validações

- Anos devem estar em ordem crescente na lista
- Faixa de anos válida
- Valores de inflação positivos

### 5. Implementar testes em `#[cfg(test)]`

**Caso 1: Básico (exemplo do requisito)**
- Input: start_year=2023, end_year=2025
- Inflações: 2023: 1,22%, 2024: 3,23%, 2025: 4,32%
- Esperado: 1.0900 (4 casas decimais)
- Cálculo: (1 + 1,22/100) * (1 + 3,23/100) * (1 + 4,32/100) = 1.012200 * 1.032300 * 1.043200 ≈ 1.09004

**Caso 2: Ano único**
- Input: start_year=2023, end_year=2023
- Esperado: 1.0122 (arredondamento aritmético)

**Caso 3: Valor ausente**
- Input: start_year=2020, end_year=2025 (2020 não existe)
- Esperado: Retorna erro `YearNotFound(2020)`

**Caso 4: Ordem invertida**
- Input: start_year=2025, end_year=2023
- Esperado: Retorna erro ou mantém ordem correta

## Verification Checklist

- [ ] `cargo build` compila sem erros
- [ ] `cargo test` passa com todos os testes de inflation
- [ ] Teste do exemplo retorna exatamente 1.0900
- [ ] lib.rs compila com nova exportação
- [ ] Erros de ano ausente são capturados e retornam Result::Err
- [ ] Código segue padrões de budget.rs (documentação, derives, validação)

## Notas de Implementação

- Usar `Decimal::from_str_exact()` para conversão de strings de inflação
- Para arredondamento: `Decimal::round_dp_round(4, RoundingStrategy::RoundCeiling)`
- Implementar `Display` e `std::error::Error` para `InflationError`
- Manter alinhamento com padrões de `budget.rs` (doc comments, testes, etc.)
